# OBridge EVM contract 

This project is built based on Hardhat, including a contract file, a test script, and a deployment script

## Before starting
```
npm install
```
or
```
yarn install
```

## Show node information
```
npx hardhat node
```

## Test
```
npx hardhat compile
npx hardhat test
```


## Deploy
```
npx hardhat compile
npx hardhat run scripts/deploy.js
```

## Deploy to GOERLI

Replace INFURA_API_KEY and GOERLI_PRIVATE_KEY in hardhat.config.js

```
npx hardhat compile
npx hardhat run scripts/deploy.js --network goerli
```