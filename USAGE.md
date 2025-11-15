Program Function:
This program takes in a configuration file containing details for each of the following: a zip file containing code, a link to users' GitHub repository, a branch of the GitHub repository to check, a list of GitHub usernames of each user, and a start and end time of the hackathon. Our program checks the following and returns an output log verifying that they are all true:

- Write some functionalities later

This program is run in the following manner:
```cargo run -- --path <path_to_json_file>```

We expect that the path leads to a JSON file in the following format:
```json
{
    "zip": "<path_to_json_file>",
	"repo": "<link_to_github_repository>",
	"usernames": ["github_username_1", "github_username_2", ..., "github_username_n"],
	"start_time": <u64_seconds_since_epoch>,
    "end_time": <u64_seconds_since_epoch>
}
```

We return an output log with the following format:
```json
{
  "first_commit_time": "Verified/Failed",
  "last_commit_time": "Verified/Failed"
}
```

If a certain time field was verified, then the status of "Verified" remains "Verified".
If a certain time field was not verified, then a failure object returns and is one of two types:

```json
{
  "first_commit_time": {
    "Failed": {
      "errorType": "GitError",
      "erorrMessage": "<git_error_message>
    }
  },
  "last_commit_time": {
    "Failed": {
      "errorType": "TimeNotInRange",
      "actualTime": 1759613329
    }
  }
}
```