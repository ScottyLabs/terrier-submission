# Terrier Submission Verifier

Rust CLI that audits hackathon submissions by validating Git metadata and running plagiarism detection with [copydetect](https://github.com/casics/CopyDetect).

## What it does
- Clones the submitted GitHub repository and checks that the first and last commits fall inside the configured time window.
- Ensures commit authors match the expected GitHub usernames.
- Fetches other public repositories for the provided usernames (respecting a size cap) and runs copydetect against the submission.
- Writes a JSON summary to `output/result.json` and, when available, copies the copydetect HTML report to `output/report.html`.

## Prerequisites
- Rust toolchain (stable).
- `copydetect` installed and on your `PATH` (`pipx install copydetect`, `pip install copydetect`, or `uv tool install copydetect`).
- Network access to GitHub; set `GITHUB_TOKEN`/`GH_TOKEN` to avoid rate limiting.

## Quick start
1. Create a config JSON file (see below for the schema).
2. Run the tool:
   ```bash
   cargo run -- --path path/to/config.json
   ```
3. Inspect `output/result.json` (and `output/report.html` if it exists).

## Config file
```json
{
  "repo": "https://github.com/example/submission",
  "usernames": ["expected_user"],
  "start_time": 1704067200,
  "end_time": 1704153600,
  "size_threshold_kb": 100000,
  "display_threshold": 0.33
}
```

- `repo`: GitHub URL of the submission repository.
- `usernames`: Expected commit authors.
- `start_time` / `end_time`: Unix seconds bounding the allowed first and last commit times.
- `size_threshold_kb` (optional, default `100000`): Total size limit (KB) of cloned comparison repos per user.
- `display_threshold` (optional, default `0.33`): Copydetect display threshold used when parsing similarity.

## Output format
`output/result.json` mirrors these shapes:
```json
{
  "metadata": {
    "first_commit_time": "Verified",
    "last_commit_time": "Verified",
    "contributors": "Verified"
  },
  "plagiarism": {
    "result": {
      "Verified": 0.08
    }
  }
}
```
Failures include structured error details (e.g., `GitError`, `TimeNotInRange`, `UsernameMismatch`), and plagiarism returns `ManualRequired` when copydetect cannot provide a score.

## Notes
- Temporary clones live in `/tmp/repo_copydetect` and are cleaned up after each run.
- Existing `output/` is replaced on every execution.
- Tests run locally via `cargo test` and do not require GitHub access.
