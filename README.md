# forked

![Release package](https://img.shields.io/github/workflow/status/jensmetzner/forked/CI)
![GitHub repo size](https://img.shields.io/github/repo-size/jensmetzner/forked)
[![GitHub license](https://img.shields.io/github/license/jensmetzner/forked.svg)](https://github.com/jensmetzner/forked/blob/master/LICENSE)
[![GitHub forks](https://img.shields.io/github/downloads/jensmetzner/forked/total)](https://github.com/JensMetzner/forked/releases/latest)

`forked` is an tool for managing online exercises that use gitlab.

## Usage

You can install forked with `cargo install forked` or download it from the [release page](https://github.com/JensMetzner/forked/releases/latest)


```
forked 0.1.2
Jens Metzner <jens.metzner@uni-konstanz.de>
`forked` is an online tool for managing exercises that use gitlab.

USAGE:
    forked.exe --gitlab-token <gitlab-token> --gitlab-api-url <gitlab-api-url> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --gitlab-api-url <gitlab-api-url>
            Gitlab api url. (Not necessary if the environment variable `GITLAB_API` is set) [env:
            GITLAB_API]

    -g, --gitlab-token <gitlab-token>
            Personal access token. (Not necessary if the environment variable `GITLAB_TOKEN` is set)
            [env: GITLAB_TOKEN]


SUBCOMMANDS:
    checkout    Runs `git checkout <branch>` for all groups
    clone       Runs `git clone <repository>` for all groups
    feedback    Either create or publish all feedback files for all groups
    help        Prints this message or the help of the given subcommand(s)
    init        Initialize a course, adding all forked repositories to `forked.yml`
    pull        Runs `git pull` for all groups
```

## Contributing to forked
To contribute to forked, follow these steps:

1. Fork this repository.
2. Create a branch: `git checkout -b <branch_name>`.
3. Make your changes and commit them: `git commit -m '<commit_message>'`
4. Push to the original branch: `git push origin <project_name>/<location>`
5. Create the pull request.

Alternatively see the GitHub documentation on [creating a pull request](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request).


### Commit messages
Please use the emojis provided in this [guideline](https://gitmoji.carloscuesta.me/) on git commit messages.
They provide an easy way to identify the purpose or intention of a commit by only looking at the emojis used.

## License
This project uses the following license: [Apache License 2.0](LICENSE)
