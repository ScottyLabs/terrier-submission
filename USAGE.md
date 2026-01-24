# Usage

## Running the verifier
1. Install dependencies: Rust toolchain and `copydetect` on your `PATH`.
2. (Recommended) Export `GITHUB_TOKEN`/`GH_TOKEN` to avoid GitHub rate limits.
3. Execute the CLI with a config file:
   ```bash
   cargo run -- --path path/to/config.json
   ```

## Config file schema
```json
{
  "repo": "https://github.com/example/submission",
  "usernames": ["github_username_1", "github_username_2"],
  "start_time": 1704067200,
  "end_time": 1704153600,
  "size_threshold_kb": 100000,
  "display_threshold": 0.33
}
```
- `repo`: GitHub URL of the submission repository.
- `usernames`: Expected commit authors.
- `start_time` / `end_time`: Unix epoch seconds bounding acceptable first/last commit times.
- `size_threshold_kb` (optional, default `100000`): Total KB of comparison repos to clone per user.
- `display_threshold` (optional, default `0.33`): Copydetect display threshold used when parsing similarity.

## What the tool does
- Clones the submission repository and verifies commit times and contributors against the provided constraints.
- Lists recent public repos for each username, filters to those created before `start_time`, and clones them (shallow) until the cumulative size cap is reached.
- Runs copydetect against the submission using those clones. If no comparison repos are available or copydetect cannot produce a report, plagiarism is marked `ManualRequired`.
- Cleans up temporary clones in `/tmp/repo_copydetect` and writes results to `output/`.

## Outputs
- `output/result.json`: Structured status for metadata (`Verified`, `Skipped`, or `Failed` with details) and plagiarism (`Verified(<decimal>)` or `ManualRequired`).
- `output/result.json` also includes `github_issues` with any invalid/private/nonexistent repo or username problems.
- `output/report.html`: The copydetect report when one was generated (copied even if the score could not be parsed).

Example success:
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
  },
  "github_issues": []
}
```

Example failure payloads mirror the metadata errors (`GitError`, `TimeNotInRange`, `UsernameMismatch`) and use `ManualRequired` for plagiarism when automated scoring is not available.
