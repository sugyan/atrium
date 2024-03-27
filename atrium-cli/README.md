# ATrium CLI

[![](https://img.shields.io/crates/v/atrium-cli)](https://crates.io/crates/atrium-cli)

CLI application for AT Protocol using ATrium API

```
Usage: atrium-cli [OPTIONS] <COMMAND>

Commands:
  login               Login (Create an authentication session)
  get-timeline        Get a view of an actor's home timeline
  get-author-feed     Get a view of an actor's feed
  get-likes           Get a list of likes for a given post
  get-reposted-by     Get a list of reposts for a given post
  get-actor-feeds     Get a list of feeds created by an actor
  get-feed            Get a view of a hydrated feed
  get-list-feed       Get a view of a specified list,
  get-follows         Get a list of who an actor follows
  get-followers       Get a list of an actor's followers
  get-lists           Get a list of the list created by an actor
  get-list            Get detailed info of a specified list
  get-profile         Get detailed profile view of an actor
  get-preferences     Get preferences of an actor
  list-notifications  Get a list of notifications
  create-post         Create a new post
  delete-post         Delete a post
  help                Print this message or the help of the given subcommand(s)

Options:
  -p, --pds-host <PDS_HOST>  [default: https://bsky.social]
  -d, --debug                Debug print
  -h, --help                 Print help
  -V, --version              Print version
```
