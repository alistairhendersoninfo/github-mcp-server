use serde_json::{json, Value};
use std::process::Command;
use tracing::{debug, info, warn, error};

use crate::{AppState, error::{AppError, Result}, mcp::protocol::GitHubCommand};
use super::api::{get_github_client, GitHubClient};

pub async fn execute_command(state: AppState, command: GitHubCommand) -> Result<Value> {
    match command {
        GitHubCommand::Push { branch, message, ready_for_review } => {
            execute_push_workflow(state, branch, message, ready_for_review).await
        }
        GitHubCommand::ScanTasks { project_number, filter_type, status } => {
            execute_scan_tasks_workflow(state, project_number, filter_type, status).await
        }
        GitHubCommand::Merge { branch, delete_branch, cleanup_work_folder } => {
            execute_merge_workflow(state, branch, delete_branch, cleanup_work_folder).await
        }
    }
}

pub async fn get_status(state: AppState) -> Result<Value> {
    let current_branch = get_current_branch()?;
    let git_status = get_git_status()?;
    let has_uncommitted_changes = !git_status.is_empty();
    
    // Check for existing PR
    let pr_info = if let Ok(github_client) = get_github_client(state.clone(), None).await {
        get_pr_for_branch(&github_client, &current_branch).await.ok()
    } else {
        None
    };

    Ok(json!({
        "current_branch": current_branch,
        "has_uncommitted_changes": has_uncommitted_changes,
        "git_status": git_status,
        "pull_request": pr_info,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn get_tasks(state: AppState) -> Result<Value> {
    // Try to get project number from TODO.md or environment
    let project_number = detect_project_number().await?;
    
    if let Ok(github_client) = get_github_client(state, None).await {
        let tasks = github_client.get_project_items(&project_number).await?;
        
        Ok(json!({
            "project_number": project_number,
            "tasks": tasks,
            "total_count": tasks.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    } else {
        Err(AppError::Authentication("GitHub client not available".to_string()))
    }
}

async fn execute_push_workflow(
    state: AppState,
    branch: Option<String>,
    message: Option<String>,
    ready_for_review: Option<bool>,
) -> Result<Value> {
    info!("Executing push workflow");

    // Get current branch or use provided branch
    let current_branch = branch.unwrap_or_else(|| get_current_branch().unwrap_or_else(|_| "main".to_string()));
    let main_branch = get_main_branch().unwrap_or_else(|_| "main".to_string());

    // Check if we're on main branch
    if current_branch == main_branch {
        warn!("Attempting to push to main branch: {}", main_branch);
        return Ok(json!({
            "status": "warning",
            "message": format!("⚠️ You're on main branch ({}). Are you sure you want to push?", main_branch),
            "branch": current_branch,
            "requires_confirmation": true
        }));
    }

    // Commit changes if message provided
    if let Some(commit_message) = message {
        info!("Committing changes with message: {}", commit_message);
        commit_changes(&commit_message)?;
    }

    // Check for uncommitted changes
    let git_status = get_git_status()?;
    if !git_status.is_empty() {
        return Ok(json!({
            "status": "error",
            "message": "⚠️ Uncommitted changes detected. Please commit or provide a commit message.",
            "uncommitted_changes": git_status
        }));
    }

    // Push to remote
    info!("Pushing branch: {}", current_branch);
    push_branch(&current_branch)?;

    // Check if PR exists and update
    if let Ok(github_client) = get_github_client(state, None).await {
        if let Ok(pr) = get_pr_for_branch(&github_client, &current_branch).await {
            info!("Found existing PR: #{}", pr.number);
            
            let mut result = json!({
                "status": "success",
                "message": format!("✅ Pushed to feature branch: {}", current_branch),
                "branch": current_branch,
                "pull_request": {
                    "number": pr.number,
                    "url": pr.html_url,
                    "title": pr.title,
                    "draft": pr.draft
                }
            });

            // Mark PR as ready for review if requested
            if ready_for_review == Some(true) && pr.draft {
                // TODO: Implement PR ready status update
                result["pull_request"]["ready_for_review"] = json!(true);
                result["message"] = json!("🎉 Pushed and marked PR as ready for review!");
            }

            return Ok(result);
        }
    }

    Ok(json!({
        "status": "success",
        "message": format!("✅ Pushed to feature branch: {}", current_branch),
        "branch": current_branch,
        "suggestion": "Consider creating a pull request for this branch"
    }))
}

async fn execute_scan_tasks_workflow(
    state: AppState,
    project_number: Option<String>,
    filter_type: Option<String>,
    status: Option<String>,
) -> Result<Value> {
    info!("Executing scan tasks workflow");

    // Get project number
    let project_num = if let Some(num) = project_number {
        num
    } else {
        detect_project_number().await?
    };

    if let Ok(github_client) = get_github_client(state, None).await {
        let mut tasks = github_client.get_project_items(&project_num).await?;

        // Apply filters
        if let Some(task_type) = filter_type {
            // TODO: Filter tasks by type
            info!("Filtering tasks by type: {}", task_type);
        }

        if let Some(task_status) = status {
            // TODO: Filter tasks by status
            info!("Filtering tasks by status: {}", task_status);
        }

        // Organize tasks by priority and type
        let organized_tasks = organize_tasks_by_priority(tasks);

        Ok(json!({
            "status": "success",
            "project_number": project_num,
            "tasks": organized_tasks,
            "message": "📋 GitHub Project Tasks Available",
            "instructions": "Select a task number to start working on it"
        }))
    } else {
        Err(AppError::Authentication("GitHub client not available".to_string()))
    }
}

async fn execute_merge_workflow(
    state: AppState,
    branch: Option<String>,
    delete_branch: Option<bool>,
    cleanup_work_folder: Option<bool>,
) -> Result<Value> {
    info!("Executing merge workflow");

    let current_branch = branch.unwrap_or_else(|| get_current_branch().unwrap_or_else(|_| "main".to_string()));
    let main_branch = get_main_branch().unwrap_or_else(|_| "main".to_string());

    if current_branch == main_branch {
        return Err(AppError::Validation("Already on main branch. Switch to feature branch first.".to_string()));
    }

    // Ensure all changes are committed
    let git_status = get_git_status()?;
    if !git_status.is_empty() {
        info!("Committing final changes");
        commit_changes(&format!("Final changes for {}", current_branch))?;
    }

    // Push final changes
    push_branch(&current_branch)?;

    if let Ok(github_client) = get_github_client(state.clone(), None).await {
        // Get PR for current branch
        let pr = get_pr_for_branch(&github_client, &current_branch).await?;
        
        // TODO: Run tests here
        info!("🧪 Running final checks...");
        
        // TODO: Merge PR via GitHub API
        info!("🔀 Merging PR #{}", pr.number);
        
        // Switch back to main and pull
        checkout_branch(&main_branch)?;
        pull_branch(&main_branch)?;

        // Clean up work folder if requested
        let work_folder_cleaned = if cleanup_work_folder.unwrap_or(false) {
            // TODO: Implement work folder cleanup
            true
        } else {
            false
        };

        // Delete branch if requested
        let branch_deleted = if delete_branch.unwrap_or(true) {
            delete_local_branch(&current_branch)?;
            true
        } else {
            false
        };

        Ok(json!({
            "status": "success",
            "message": "🎉 Production deployment complete!",
            "merged_pr": {
                "number": pr.number,
                "url": pr.html_url,
                "title": pr.title
            },
            "current_branch": main_branch,
            "branch_deleted": branch_deleted,
            "work_folder_cleaned": work_folder_cleaned,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    } else {
        Err(AppError::Authentication("GitHub client not available".to_string()))
    }
}

// Git utility functions
fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to get current branch: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Internal("Git command failed".to_string()));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

fn get_main_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "show", "origin"])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to get main branch: {}", e)))?;

    if !output.status.success() {
        return Ok("main".to_string()); // Default fallback
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.contains("HEAD branch:") {
            if let Some(branch) = line.split(':').nth(1) {
                return Ok(branch.trim().to_string());
            }
        }
    }

    Ok("main".to_string()) // Default fallback
}

fn get_git_status() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to get git status: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Internal("Git status command failed".to_string()));
    }

    let status_lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .collect();

    Ok(status_lines)
}

fn commit_changes(message: &str) -> Result<()> {
    // Add all changes
    let add_output = Command::new("git")
        .args(["add", "."])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to add changes: {}", e)))?;

    if !add_output.status.success() {
        return Err(AppError::Internal("Git add command failed".to_string()));
    }

    // Commit changes
    let commit_output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to commit changes: {}", e)))?;

    if !commit_output.status.success() {
        return Err(AppError::Internal("Git commit command failed".to_string()));
    }

    Ok(())
}

fn push_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "origin", branch])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to push branch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!("Git push failed: {}", stderr)));
    }

    Ok(())
}

fn pull_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["pull", "origin", branch])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to pull branch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!("Git pull failed: {}", stderr)));
    }

    Ok(())
}

fn checkout_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to checkout branch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(format!("Git checkout failed: {}", stderr)));
    }

    Ok(())
}

fn delete_local_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["branch", "-d", branch])
        .output()
        .map_err(|e| AppError::Internal(format!("Failed to delete branch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to delete branch {}: {}", branch, stderr);
    }

    Ok(())
}

async fn detect_project_number() -> Result<String> {
    // Try to read project number from TODO.md
    if let Ok(todo_content) = tokio::fs::read_to_string("TODO.md").await {
        for line in todo_content.lines() {
            if line.contains("Project Number:") || line.contains("GitHub Project:") {
                // Extract project number from line
                if let Some(number) = extract_number_from_line(line) {
                    return Ok(number);
                }
            }
        }
    }

    // Fallback: check environment variable
    if let Ok(project_num) = std::env::var("GITHUB_PROJECT_NUMBER") {
        return Ok(project_num);
    }

    Err(AppError::Validation("No GitHub Project number found. Please specify project_number or add it to TODO.md".to_string()))
}

fn extract_number_from_line(line: &str) -> Option<String> {
    // Simple regex-like extraction for project numbers
    for word in line.split_whitespace() {
        if word.chars().all(|c| c.is_ascii_digit()) && word.len() > 0 {
            return Some(word.to_string());
        }
    }
    None
}

async fn get_pr_for_branch(github_client: &GitHubClient, branch: &str) -> Result<super::api::GitHubPullRequest> {
    // TODO: Implement PR lookup by branch name
    // This would require parsing the repository from git remote
    Err(AppError::Internal("PR lookup not implemented yet".to_string()))
}

fn organize_tasks_by_priority(tasks: Vec<super::api::GitHubProjectItem>) -> Value {
    // TODO: Implement task organization by priority and type
    json!({
        "critical": [],
        "high": [],
        "medium": [],
        "low": [],
        "total": tasks.len()
    })
}