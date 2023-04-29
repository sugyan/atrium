# CLI

```
Usage: atrium-cli [OPTIONS] <COMMAND>

Commands:
  create-record        Create a new record (post, repost, block)
  create-app-password  Create a new app password
  delete-record        Delete record
  get-session          Get current session info
  get-profile          Get a profile of an actor (default: current session)
  get-record           Get record
  get-timeline         Get timeline
  get-follows          Get following of an actor (default: current session)
  get-followers        Get followers of an actor (default: current session)
  get-author-feed      Get a feed of an author (default: current session)
  get-post-thread      Get a post thread
  get-blocks           Get a list of blocking actors
  list-app-passwords   List app passwords
  revoke-app-password  Revoke an app password
  help                 Print this message or the help of the given subcommand(s)

Options:
  -p, --pds-host <PDS_HOST>  [default: https://bsky.social]
  -c, --config <CONFIG>      Path to config file [default: config.toml]
  -d, --debug                Debug print
  -h, --help                 Print help
  -V, --version              Print version
```

## sub commands

```
Create a new record (post, repost, block)

Usage: atrium-cli create-record <COMMAND>

Commands:
  post    Create a post
  repost  Create a repost
  block   Block an actor
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
  -i, --image <IMAGE>  image files
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

```
Block an actor

Usage: atrium-cli create-record block <DID>

Arguments:
  <DID>  DID of an actor to block

Options:
  -h, --help  Print help
```
