# CLI

```
Usage: atrium-cli [OPTIONS] <COMMAND>

Commands:
  create-record    Create a new record (post, repost)
  get-session      Get current session info
  get-profile      Get a profile of an actor (default: current session)
  get-record       Get record
  get-timeline     Get timeline
  get-follows      Get following of an actor (default: current session)
  get-followers    Get followers of an actor (default: current session)
  get-author-feed  Get a feed of an author (default: current session)
  get-post-thread  Get a post thread
  help             Print this message or the help of the given subcommand(s)

Options:
  -p, --pds-host <PDS_HOST>  [default: https://bsky.social]
  -c, --config <CONFIG>      Path to config file [default: config.toml]
  -d, --debug                Debug print
  -h, --help                 Print help
  -V, --version              Print version
```

## sub commands

```
Create a new record (post, repost)

Usage: atrium-cli create-record <COMMAND>

Commands:
  post    Create a post
  repost  Create a repost
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
Create a post

Usage: atrium-cli create-record post [OPTIONS] <TEXT>

Arguments:
  <TEXT>  Text of the post

Options:
  -r, --reply <REPLY>  URI of the post to reply to
  -h, --help           Print help
```

```
Create a repost

Usage: atrium-cli create-record repost <URI>

Arguments:
  <URI>  URI of the post to repost

Options:
  -h, --help  Print help
```
