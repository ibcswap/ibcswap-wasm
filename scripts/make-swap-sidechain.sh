HOME1="/Users/ghostprince/.wasmd1"
HOME2="/Users/ghostprince/.wasmd2"
KEY="bob"
CHAIN_ID="sidechain_7070-1"
CHANNEL_ID="channel-6"
RECEIVER="wasm1fll0djfrcpkhszxzf4lfg6fp7eu6ywxqcn6pnx"

sidechaind tx ibc-swap make $CHANNEL_ID 100aside $RECEIVER 100stake --from $KEY --keyring-backend test --chain-id $CHAIN_ID --gas-prices 0.01aside --gas auto --gas-adjustment 1.2 --trace --packet-timeout-height "0-9999993" --packet-timeout-timestamp "1693399799000000000"