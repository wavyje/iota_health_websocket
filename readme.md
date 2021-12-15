#                                               ABOUT
### Websocket, containing the IOTA Streams Logic for the corresponding app in repository: 
#                                            INSTALLATION
### 1. install rustup including cargo
### 
### 2. if it is the first start (files "author_state.bin", "subscriber_state.bin", "subscriber_reading_state.bin", 
###    "channel_address.bin" do not exist) or you want to reset the channel, type 'cargo run initiate'
###
### 3. after initialization the server can simply be started with 'cargo run'
###
### Known bugs (due to IOTA Streams being in alpha state):
### -when publishing the first keyload on the branch the office client will disconnect due to failed heartbeat
### -messages will be pruned after 7 days, the channel will then not work correctly
### -subscriber instances that are not included in the keyload message can read the signed messages at the moment