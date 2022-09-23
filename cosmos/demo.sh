#!/bin/env bash

echo "\n\n==> Deploy"
RES=$(wasmd tx wasm store cosmos_atomic_swap.wasm --from wallet --node https://rpc.malaga-420.cosmwasm.com:443 --chain-id malaga-420 --gas-prices 0.25umlg --gas auto --gas-adjustment 1.3 -y --output json -b block)
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Code ID: ${CODE_ID}"
INIT='{"platform": "wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","fee":{"amount":"1","denom":"umlg"}}'
wasmd tx wasm instantiate $CODE_ID "$INIT" --fees 10000umlg --node https://rpc.malaga-420.cosmwasm.com:443 --chain-id malaga-420 --from wallet --label "atomic swap" -y --no-admin

sleep 6

wasmd query wasm list-contract-by-code $CODE_ID --node https://rpc.malaga-420.cosmwasm.com:443 --output json
CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID --node https://rpc.malaga-420.cosmwasm.com:443 --output json | jq -r '.contracts[ -1]')
echo "Contract: $CONTRACT"

echo "\n\n==> Contract information"
wasmd query wasm contract $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443
echo "\n\n==> Contract balance"
wasmd query bank balances $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq
echo "\n\n==> Contract status"
wasmd query wasm contract-state all $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443  -o json | jq

echo "\n\n==> Alice fund (transfer 1)"
FUND1MSG='{"fund":{"sender":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","receiver":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","coin": {"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1}}'
wasmd tx wasm execute $CONTRACT "$FUND1MSG" --node https://rpc.malaga-420.cosmwasm.com:443 --fees 10000umlg --amount 101umlg --chain-id malaga-420 --from wallet -y

sleep 6

echo "\n\n==> Contract balance"
wasmd query bank balances $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq
echo "\n\n==> Contract status"
wasmd query wasm contract-state all $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq

sleep 6

echo "\n\n==> Bob query transfer 1"
TRANSFER1_QUERY='{"sender":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","receiver":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","coin": {"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1}'
wasmd query wasm contract-state smart $CONTRACT "$TRANSFER1_QUERY" --node https://rpc.malaga-420.cosmwasm.com:443 --output json

echo "\n\n==> Bob fund (transfer 2)"
FUND2MSG='{"fund":{"sender":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","receiver":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","coin": {"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1}}'
wasmd tx wasm execute $CONTRACT "$FUND2MSG" --node https://rpc.malaga-420.cosmwasm.com:443 --fees 10000umlg --amount 101umlg --chain-id malaga-420 --from wallet2 -y

sleep 6

echo "\n\n==> Contract balance"
wasmd query bank balances $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq
echo "\n\n==> Contract status"
wasmd query wasm contract-state all $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq

sleep 6

echo "\n\n==> Alice query transfer 2"
TRANSFER2_QUERY='{"sender":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","receiver":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","coin": {"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1}'
wasmd query wasm contract-state smart $CONTRACT "$TRANSFER2_QUERY" --node https://rpc.malaga-420.cosmwasm.com:443 --output json

echo "\n\n==> Alice confirm (transfer 2)"
CONFIRM2MSG='{"confirm":[{"sender":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","receiver":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","coin":{"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1},[115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115]]}'
wasmd tx wasm execute $CONTRACT "$CONFIRM2MSG" --node https://rpc.malaga-420.cosmwasm.com:443 --fees 10000umlg --chain-id malaga-420 --from wallet2 -y

sleep 6

echo "\n\n==> Contract balance"
wasmd query bank balances $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq
echo "\n\n==> Contract status"
wasmd query wasm contract-state all $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq

sleep 6

echo "\n\n==> Bob query transfer 2"
TRANSFER2_QUERY='{"sender":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","receiver":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","coin": {"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1}'
wasmd query wasm contract-state smart $CONTRACT "$TRANSFER2_QUERY" --node https://rpc.malaga-420.cosmwasm.com:443 --output json

echo "\n\n==> Bob confirm (transfer 1)"
CONFIRM1MSG='{"confirm":[{"sender":"wasm13purjga76lucrpv9zsh6fp3w92wezmjd2jdw6v","receiver":"wasm1y7n3fe6ppa62whq3z5gyh26q30xxhu0gnrzyj9","coin":{"amount":"100","denom":"umlg"},"hashlock":[165,152,132,76,216,153,182,114,45,89,20,251,170,95,204,77,214,166,43,58,171,243,206,181,109,46,63,177,197,13,234,154],"timelock":1},[115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115,115]]}'
wasmd tx wasm execute $CONTRACT "$CONFIRM1MSG" --node https://rpc.malaga-420.cosmwasm.com:443 --fees 10000umlg --chain-id malaga-420 --from wallet2 -y

sleep 6

echo "\n\n==> Contract balance"
wasmd query bank balances $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq
echo "\n\n==> Contract status"
wasmd query wasm contract-state all $CONTRACT --node https://rpc.malaga-420.cosmwasm.com:443 -o json | jq
