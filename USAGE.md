Program Function:
This program takes in a configuration file containing details for each of the following: a zip file containing code, a link to users' GitHub repository, a branch of the GitHub repository to check, a list of GitHub usernames of each user, and a start and end time of the hackathon. Our program checks the following and returns an output log verifying that they are all true:

- Metadata verification: First commit time, last commit time, and contributors match the expected constraints
- Plagiarism detection: Similarity percentage between the submitted code and other public repositories

This program is run in the following manner:
```cargo run -- --path <path_to_json_file>```

We expect that the path leads to a JSON file in the following format:
```json
{
    "zip": "<path_to_json_file>",
	"repo": "<link_to_github_repository>",
	"usernames": ["github_username_1", "github_username_2", ..., "github_username_n"],
	"start_time": <u64_seconds_since_epoch>,
    "end_time": <u64_seconds_since_epoch>,
    "size_threshold_kb": 100000
}
```

Note: `size_threshold_kb` is optional and defaults to 100,000 KB (~100MB). This limits the cumulative size of repositories cloned for plagiarism detection.

We return an output log with the following format:
```json
{
  "metadata": {
    "first_commit_time": "Verified/Skipped",
    "last_commit_time": "Verified/Skipped",
    "contributors": "Verified/Skipped"
  },
  "plagiarism": {
    "result": {
      "Verified": <similarity_percentage_as_decimal>
    }
  }
}
```

If a certain metadata field was verified, then the status of "Verified" remains "Verified".
If a certain metadata field was not checked (no constraint provided), the status is "Skipped".
If a certain metadata field was not verified, then a failure object returns and is one of several types:

```json
{
  "metadata": {
    "first_commit_time": {
      "Failed": {
        "errorType": "GitError",
        "errorMessage": "<git_error_message>"
      }
    },
    "last_commit_time": {
      "Failed": {
        "errorType": "TimeNotInRange",
        "actualTime": 1759613329
      }
    },
    "contributors": {
      "Failed": {
        "errorType": "UsernameMismatch",
        "unexpectedContributors": ["unauthorized_user1", "unauthorized_user2"]
      }
    }
  },
  "plagiarism": {
    "result": {
      "Verified": 0.15
    }
  }
}
```

For plagiarism verification:
- `Verified(<percentage>)`: Contains a decimal similarity percentage (e.g., 0.15 = 15% similarity)
- `PrerequisitesNotMet`: The copydetect tool is not installed or the HTML report was not generated

Example output with successful verification:
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