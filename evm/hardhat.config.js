require("@nomicfoundation/hardhat-toolbox");

// Go to https://infura.io/, sign up, create
// a new App in its dashboard, and replace "KEY" with its key
const INFURA_API_KEY = "KEY";

// Replace this private key with your Goerli account private key
// To export your private key from Metamask, open Metamask and
// go to Account Details > Export Private Key
// Be aware of NEVER putting real Ether into testing accounts
const GOERLI_PRIVATE_KEY = "YOUR GOERLI PRIVATE KEY";

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.17",
  // networks: {
  //   goerli: {
  //     url: `https://rinkeby.infura.io/v3/${INFURA_API_KEY}`,
  //     accounts: [`0x${GOERLI_PRIVATE_KEY}`]
  //   }
  // }
  networks: {
    ht_test: {
      url: `https://http-testnet.hecochain.com`,
      accounts: [`0x81d2d54b40141982d0c77ba4569a821f929ed5343d1cce0a49f705c1a0c18f45`]
    },
    bsc_test: {
      url: 'https://blockchain2.byte-trade.com:31267/bsc-archive-testnet',
      // chainId: 97,
      // gasPrice: 20000000000,
      accounts: ['0x13b225fbc8e4e7e6f49395b9edb8aa9a32fb9e3c37c3a37df3f998fcc17e36fa']
    }
  }
};
