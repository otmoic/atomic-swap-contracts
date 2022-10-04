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
};
