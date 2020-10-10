A fully asynchronous, [futures](https://github.com/rust-lang/futures-rs)-enabled client
library for Rust based on [Dcrd](https://github.com/decred/dcrd) with true asynchronous programming.

## Ported Functionalities
- [ ] RPC Client
    - [ ] Websocket Notification
        - [x] Notify On Block Connected
        - [x] Notify On Block Disconnected
        - [x] Notify On Winning Tickets
        - [x] Notify On Work
        - [x] Notify On Transaction Accepted
        - [x] Notify On Transaction Accepted Verbose
        - [x] Notify On Stake Difficulty
        - [x] Notify On Reorganization
        - [ ] Notify On Relevant Transaction Spent
        - [ ] Notify On Spent And Missed Tickets
        - [ ] Notify On New Tickets

    - [ ] RPC Commands
        |        Command      |  Websocket supported |   HTTP Supported   |
        |:-------------------:|:--------------------:|:------------------:|
        | Get Blockchain Info |  :white_check_mark:  | :white_check_mark: |
        | Get Block Count     |  :white_check_mark:  | :white_check_mark: |
        | Get Block Hash      |  :white_check_mark:  | :white_check_mark: |

- [ ] Chaincfg
    - [x] Chain hash

- [ ] Dcr Utilities
    - [x] Get App Data Directory
    - [x] DCR Amount
