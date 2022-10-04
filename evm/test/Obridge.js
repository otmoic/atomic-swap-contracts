const {
    time,
    loadFixture,
  } = require("@nomicfoundation/hardhat-network-helpers");
const { anyValue } = require("@nomicfoundation/hardhat-chai-matchers/withArgs");
const { expect } = require("chai");

describe('OBridge', function () {
    async function deployOBridge () {
        const [owner, otherAccount, user, lp] = await ethers.getSigners();

        const OBridge = await hre.ethers.getContractFactory("OBridge");
        const obridge = await OBridge.deploy()
        await obridge.deployed();

        return {obridge, owner, otherAccount, user, lp}
    }
    async function deployTestERC20Src () {
        const [owner, otherAccount] = await ethers.getSigners();

        const initialSupply = '10000000000000000000000'

        const TestERC20 = await hre.ethers.getContractFactory("TestERC20Src");
        const tercSrc = await TestERC20.deploy(initialSupply)
        await tercSrc.deployed();

        return {tercSrc, owner, otherAccount, initialSupply}
    }
    async function deployTestERC20Dst () {
        const [owner, otherAccount] = await ethers.getSigners();

        const initialSupply = '10000000000000000000000'

        const TestERC20 = await hre.ethers.getContractFactory("TestERC20Dst");
        const tercDst = await TestERC20.deploy(initialSupply)
        await tercDst.deployed();

        return {tercDst, owner, otherAccount, initialSupply}
    }

    let cache = {}

    //TODO Fill use case
    // describe('confirm', function () {
        
    //     describe('out', function () {
        
    //     })
        
    //     describe('in', function () {
        
    //     })
    // })

    // describe('business', function () {
        
    //     describe('ERC20 -> ERC20', function () {
    //         it("TokenA(ERC20) -> TokenB(ERC20)", async function () {
    //             let tercSrc = cache.tercSrc
    //             // let tercDst = cache.tercDst
    //             const {tercDst} = await loadFixture(deployTestERC20Dst);
    //             const {obridge, owner, otherAccount, user, lp} = await loadFixture(deployOBridge);

    //             let token_amount_src = '1000000000000000000'
    //             let token_amount_dst = '1000000000000000'
    //             let eth_amount       = '0'

    //             let srcTransferId = new Array(32).fill(3)
    //             let hashlock = new Array(32).fill(1)
    //             let preimage = new Array(32).fill(2)
    //             let timelock = new Date().getTime() + 10000
    //             let srcChainId = '60'
    //             let dstChainId = '60'
    //             let bidId = '1'

    //             // console.log('tercSrc:')
    //             // console.log(tercSrc)

    //             // await expect(tercSrc.transfer(user.address, token_amount_src))
    //             // .to.emit(tercSrc, "Transfer")
    //             // .withArgs(
    //             //     owner.address,
    //             //     user.address,
    //             //     token_amount_src
    //             // )

    //             console.log(owner.address)

    //             // const ownerBalance = await tercDst.balanceOf(owner.address);
    //             // console.log(ownerBalance)

    //             await expect(tercDst.transfer(lp.address, token_amount_dst))
    //             .to.emit(tercDst, "Transfer")
    //             .withArgs(
    //                 owner.address,
    //                 lp.address,
    //                 token_amount_dst
    //             )

    //             // await expect(obridge.connect(user).transferOut(
    //             //     user.address,
    //             //     lp.address,
    //             //     tercSrc.address,
    //             //     token_amount_src,
    //             //     hashlock,
    //             //     timelock,
    //             //     dstChainId,
    //             //     owner.address,
    //             //     bidId,
    //             //     tercDst.address,
    //             //     token_amount_dst
    //             // ))
    //             // .to.emit(tercSrc, "Transfer")
    //             // .withArgs(
    //             //     user.address,
    //             //     obridge.address,
    //             //     token_amount_src
    //             // )

    //             // await expect(obridge.connect(lp).transferIn(
    //             //     lp.address,
    //             //     user.address,
    //             //     tercDst.address,
    //             //     token_amount_dst,
    //             //     eth_amount,
    //             //     hashlock,
    //             //     timelock,
    //             //     srcChainId,
    //             //     srcTransferId,
    //             //     {value: eth_amount}
    //             // ))

    //             // await expect(obridge.connect(user).confirm(
    //             //     user.address,       // address _sender,
    //             //     lp.address,         // address _receiver,
    //             //     tercSrc.address,    // address _token,
    //             //     token_amount_src,   // uint256 _token_amount,
    //             //     eth_amount,         // uint256 _eth_amount,
    //             //     hashlock,           // bytes32 _hashlock,
    //             //     timelock,           // uint64 _timelock,
    //             //     preimage            // bytes32 _preimage
    //             // ))

    //             // await expect(obridge.connect(lp).confirm(
    //             //     lp.address,
    //             //     user.address,
    //             //     tercDst.address,
    //             //     token_amount_dst,
    //             //     eth_amount,
    //             //     hashlock,
    //             //     timelock,
    //             //     preimage
    //             // ))

    //         })
    //     })

    //     describe('Native Token -> ERC20', function () {
        
    //     })

    //     describe('ERC20 -> Native Token', function () {
        
    //     })
    // })

    // describe('fee', function () {
    //     describe('out', function () {
        
    //     })

    //     describe('in', function () {
    //         describe('Native Token', function () {
        
    //         })

    //         describe('ERC20', function () {
        
    //         })
    //     })
    // })

    // describe('owner', function () {
    //     describe('first owner', function () {
        
    //     })
    //     describe('change', function () {
        
    //     })
    //     describe('no approve', function () {
        
    //     })
    //     describe('owner permission', function () {
        
    //     })
    // })

    // describe('error', function () {

    // })
})
