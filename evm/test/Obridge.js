const {
    time,
    loadFixture,
  } = require("@nomicfoundation/hardhat-network-helpers");
const { anyValue } = require("@nomicfoundation/hardhat-chai-matchers/withArgs");
const { expect } = require("chai");

describe('OBridge', function () {
    async function deployOBridge () {
        const [owner, otherAccount] = await ethers.getSigners();

        const OBridge = await hre.ethers.getContractFactory("OBridge");
        const obridge = await OBridge.deploy()
        await obridge.deployed();

        return {obridge, owner, otherAccount}
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
    describe('event', function () {
        
        describe('in', function () {
            it("Should emit an event on transferIn", async function () {
                const {obridge, owner, otherAccount} = await loadFixture(deployOBridge);
                const {tercSrc} = await loadFixture(deployTestERC20Src);
                cache.tercSrc = tercSrc
    
                let hashlock = new Array(32).fill(1)
                let timelock = new Date().getTime() + 10000
                let srcChainId = '60'
                let srcTransferId = new Array(32).fill(3)
                
    
                let token_amount    = '1000000000000000000'
                let eth_amount      = '100000000000000000'
                
                console.log('obridge address:', obridge.address)
                console.log('tercSrc address:', tercSrc.address)
                
                await expect(tercSrc.approve(obridge.address, token_amount))
    
                await expect(obridge.transferIn(
                    owner.address,              // address _sender,
                    otherAccount.address,       // address _dstAddress,
                    tercSrc.address,            // address _token,
                    token_amount,               // uint256 _token_amount,
                    eth_amount,                 // uint256 _eth_amount,
                    hashlock,                   // bytes32 _hashlock,
                    timelock,                   // uint64 _timelock,
                    srcChainId,                 // uint64 _srcChainId,
                    srcTransferId,              // bytes32 _srcTransferId
                    {value: eth_amount}
                ))
                .to.emit(obridge, "LogNewTransferIn")
                .withArgs(
                    anyValue,                   // bytes32 transferId,
                    owner.address,              // address sender,
                    otherAccount.address,       // address receiver,
                    tercSrc.address,            // address token,
                    token_amount,               // uint256 token_amount,
                    eth_amount,                 // uint256 eth_amount,
                    anyValue,                   // bytes32 hashlock, // hash of the preimage
                    timelock,                   // uint64 timelock, // UNIX timestamp seconds - locked UNTIL this time
                    srcChainId,                 // uint64 srcChainId,
                    anyValue,                   // bytes32 srcTransferId // outbound transferId at src chain
                )
                
            })
        })

        describe('out', function () {
            it("Should emit an event on transferOut", async function () {
                const {obridge, owner, otherAccount} = await loadFixture(deployOBridge);
                let tercSrc = cache.tercSrc
                const {tercDst} = await loadFixture(deployTestERC20Dst);
    
                let token_amount        = '1000000000000000000'
                let token_amount_dst    = '1000000000000000'
    
                let hashlock = new Array(32).fill(1)
                let timelock = new Date().getTime() + 10000
                let dstChainId = '60'
                let bidId = '1'
    
                await expect(tercSrc.approve(obridge.address, token_amount))
    
                await expect(obridge.transferOut(
                    owner.address,              // address _sender,
                    otherAccount.address,       // address _bridge,
                    tercSrc.address,               // address _token,
                    token_amount,               // uint256 _amount,
                    hashlock,                   // bytes32 _hashlock,
                    timelock,                   // uint64 _timelock,
                    dstChainId,                 // uint64 _dstChainId,
                    owner.address,              // address _dstAddress,
                    bidId,                      // uint64 _bidId,
                    tercDst.address,            // uint256 _tokenDst,
                    token_amount_dst            // uint256 _amountDst
                ))
                .to.emit(obridge, "LogNewTransferOut")
                .withArgs(
                    anyValue,                   // bytes32 transferId,
                    owner.address,              // address sender,
                    otherAccount.address,       // address receiver,
                    tercSrc.address,               // address token,
                    token_amount,               // uint256 amount,
                    anyValue,                   // bytes32 hashlock, // hash of the preimage
                    timelock,                   // uint64 timelock, // UNIX timestamp seconds - locked UNTIL this time
                    dstChainId,                 // uint64 dstChainId,
                    owner.address,              // address dstAddress,
                    bidId,                      // uint64 bidId,
                    tercDst.address,            // uint256 tokenDst,
                    token_amount_dst            // uint256 amountDst
                )
            })
        })


    })

    //TODO Fill use case
    // describe('confirm', function () {
        
    //     describe('out', function () {
        
    //     })
        
    //     describe('in', function () {
        
    //     })
    // })

    // describe('business', function () {
        
    //     describe('ERC20 -> ERC20', function () {
        
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
