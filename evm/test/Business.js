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

    describe('business', function () {
        
        describe('ERC20 -> ERC20', function () {
            it("TokenA(ERC20) -> TokenB(ERC20)", async function () {
                
                const {tercSrc} = await loadFixture(deployTestERC20Src);
                const {tercDst} = await loadFixture(deployTestERC20Dst);
                const {obridge, owner, otherAccount, user, lp} = await loadFixture(deployOBridge);

                Object.assign(cache, {
                    tercSrc,
                    tercDst,
                    obridge,
                    owner,
                    otherAccount,
                    user,
                    lp
                })

                let token_amount_src = '1000000000000000000'
                let token_amount_dst = '1000000000000000'
                let eth_amount       = '0'

                let srcTransferId = new Array(32).fill(3)
                let preimage = new Array(32).fill(2)
                let timelock = new Date().getTime() + 10000
                let srcChainId = '60'
                let dstChainId = '60'
                let bidId = '1'

                let hashlock = ethers.utils.keccak256(ethers.utils.solidityPack(['bytes32'], [preimage]))

                await expect(tercSrc.transfer(user.address, token_amount_src))
                .to.emit(tercSrc, "Transfer")
                .withArgs(
                    owner.address,
                    user.address,
                    token_amount_src
                )

                await expect(tercDst.transfer(lp.address, token_amount_dst))
                .to.emit(tercDst, "Transfer")
                .withArgs(
                    owner.address,
                    lp.address,
                    token_amount_dst
                )

                await expect(tercSrc.connect(user).approve(obridge.address, token_amount_src))
                await expect(obridge.connect(user).transferOut(
                    user.address,
                    lp.address,
                    tercSrc.address,
                    token_amount_src,
                    hashlock,
                    timelock,
                    dstChainId,
                    owner.address,
                    bidId,
                    tercDst.address,
                    token_amount_dst
                ))
                .to.emit(tercSrc, "Transfer")
                .withArgs(
                    user.address,
                    obridge.address,
                    token_amount_src
                )

                await expect(tercDst.connect(lp).approve(obridge.address, token_amount_dst))
                await expect(obridge.connect(lp).transferIn(
                    lp.address,
                    user.address,
                    tercDst.address,
                    token_amount_dst,
                    eth_amount,
                    hashlock,
                    timelock,
                    srcChainId,
                    srcTransferId,
                    {value: eth_amount}
                ))
                .to.emit(tercDst, "Transfer")
                .withArgs(
                    lp.address,
                    obridge.address,
                    token_amount_dst
                )

                await expect(obridge.connect(user).confirm(
                    user.address,       // address _sender,
                    lp.address,         // address _receiver,
                    tercSrc.address,    // address _token,
                    token_amount_src,   // uint256 _token_amount,
                    eth_amount,         // uint256 _eth_amount,
                    hashlock,           // bytes32 _hashlock,
                    timelock,           // uint64 _timelock,
                    preimage            // bytes32 _preimage
                ))
                .to.emit(tercSrc, "Transfer")
                .withArgs(
                    obridge.address,
                    lp.address,
                    token_amount_src
                )

                await expect(obridge.connect(lp).confirm(
                    lp.address,
                    user.address,
                    tercDst.address,
                    token_amount_dst,
                    eth_amount,
                    hashlock,
                    timelock,
                    preimage
                ))
                .to.emit(tercDst, "Transfer")
                .withArgs(
                    obridge.address,
                    user.address,
                    token_amount_dst
                )

            })
        })

        describe('Native Token -> ERC20', function () {
            it("Native Token -> TokenB(ERC20)", async function () {
                const {tercSrc} = cache
                const {tercDst} = cache
                const {obridge, owner, otherAccount, user, lp} = cache

                let token_amount_src = '1000000000000000000'
                let token_amount_dst = '1000000000000000'

                let eth_amount       = '0'

                let srcTransferId = new Array(32).fill(3)
                let preimage = new Array(32).fill(2)
                let timelock = new Date().getTime() + 10000
                let srcChainId = '60'
                let dstChainId = '60'
                let bidId = '2'
                let nativeTokenAddress = "0x0000000000000000000000000000000000000000"

                let hashlock = ethers.utils.keccak256(ethers.utils.solidityPack(['bytes32'], [preimage]))
                
                await expect(tercDst.transfer(lp.address, token_amount_dst))
                .to.emit(tercDst, "Transfer")
                .withArgs(
                    owner.address,
                    lp.address,
                    token_amount_dst
                )
                
                await expect(obridge.connect(user).transferOut(
                    user.address,               // address _sender,
                    lp.address,                 // address _bridge,
                    nativeTokenAddress,         // address _token,
                    token_amount_src,           // uint256 _amount,
                    hashlock,                   // bytes32 _hashlock,
                    timelock,                   // uint64 _timelock,
                    dstChainId,                 // uint64 _dstChainId,
                    user.address,               // address _dstAddress,
                    bidId,                      // uint64 _bidId,
                    tercDst.address,            // uint256 _tokenDst,
                    token_amount_dst,           // uint256 _amountDst
                    {
                        value: token_amount_src
                    }
                ))
                .to.emit(obridge, 'LogNewTransferOut')
                .and.changeEtherBalance(obridge, token_amount_src)

                await expect(tercDst.connect(lp).approve(obridge.address, token_amount_dst))
                await expect(obridge.connect(lp).transferIn(
                    lp.address,                 // address _sender,
                    user.address,               // address _dstAddress,
                    tercDst.address,            // address _token,
                    token_amount_dst,           // uint256 _token_amount,
                    eth_amount,                 // uint256 _eth_amount,
                    hashlock,                   // bytes32 _hashlock,
                    timelock,                   // uint64 _timelock,
                    srcChainId,                 // uint64 _srcChainId,
                    srcTransferId,              // bytes32 _srcTransferId
                    {value: eth_amount}
                ))
                .to.emit(obridge, "LogNewTransferIn")
                .and.emit(tercDst, "Transfer")
                .withArgs(
                    lp.address,
                    obridge.address,
                    token_amount_dst
                )


                await expect(obridge.connect(user).confirm(
                    user.address,       // address _sender,
                    lp.address,         // address _receiver,
                    nativeTokenAddress, // address _token,
                    token_amount_src,   // uint256 _token_amount,
                    eth_amount,         // uint256 _eth_amount,
                    hashlock,           // bytes32 _hashlock,
                    timelock,           // uint64 _timelock,
                    preimage            // bytes32 _preimage
                ))
                .to.changeEtherBalance(lp, token_amount_src)

                await expect(obridge.connect(lp).confirm(
                    lp.address,
                    user.address,
                    tercDst.address,
                    token_amount_dst,
                    eth_amount,
                    hashlock,
                    timelock,
                    preimage
                ))
                .to.emit(tercDst, "Transfer")
                .withArgs(
                    obridge.address,
                    user.address,
                    token_amount_dst
                )
            })
        })

        describe('ERC20 -> Native Token', function () {
            it("TokenA(ERC20) -> Native Token", async function () {
                const {tercSrc} = cache
                const {tercDst} = cache
                const {obridge, owner, otherAccount, user, lp} = cache

                let token_amount_src = '1000000000000000000'
                let token_amount_dst = '1000000000000000'

                let eth_amount       = '0'

                let srcTransferId = new Array(32).fill(3)
                let preimage = new Array(32).fill(2)
                let timelock = new Date().getTime() + 10000
                let srcChainId = '60'
                let dstChainId = '60'
                let bidId = '3'
                let nativeTokenAddress = "0x0000000000000000000000000000000000000000"

                let hashlock = ethers.utils.keccak256(ethers.utils.solidityPack(['bytes32'], [preimage]))

                await expect(tercSrc.transfer(user.address, token_amount_src))
                .to.emit(tercSrc, "Transfer")
                .withArgs(
                    owner.address,
                    user.address,
                    token_amount_src
                )

                await expect(tercSrc.connect(user).approve(obridge.address, token_amount_src))
                await expect(obridge.connect(user).transferOut(
                    user.address,               // address _sender,
                    lp.address,                 // address _bridge,
                    tercSrc.address,            // address _token,
                    token_amount_src,           // uint256 _amount,
                    hashlock,                   // bytes32 _hashlock,
                    timelock,                   // uint64 _timelock,
                    dstChainId,                 // uint64 _dstChainId,
                    user.address,               // address _dstAddress,
                    bidId,                      // uint64 _bidId,
                    nativeTokenAddress,         // uint256 _tokenDst,
                    token_amount_dst,           // uint256 _amountDst
                ))
                .to.emit(obridge, 'LogNewTransferOut')
                .and.emit(tercSrc, "Transfer")
                .withArgs(
                    user.address,
                    obridge.address,
                    token_amount_src
                )    

                await expect(obridge.connect(lp).transferIn(
                    lp.address,                 // address _sender,
                    user.address,               // address _dstAddress,
                    nativeTokenAddress,         // address _token,
                    token_amount_dst,           // uint256 _token_amount,
                    eth_amount,                 // uint256 _eth_amount,
                    hashlock,                   // bytes32 _hashlock,
                    timelock,                   // uint64 _timelock,
                    srcChainId,                 // uint64 _srcChainId,
                    srcTransferId,              // bytes32 _srcTransferId
                    {value: token_amount_dst}
                ))
                .to.emit(obridge, "LogNewTransferIn")
                .and.changeEtherBalance(obridge, token_amount_dst)


                await expect(obridge.connect(user).confirm(
                    user.address,       // address _sender,
                    lp.address,         // address _receiver,
                    tercSrc.address,    // address _token,
                    token_amount_src,   // uint256 _token_amount,
                    eth_amount,         // uint256 _eth_amount,
                    hashlock,           // bytes32 _hashlock,
                    timelock,           // uint64 _timelock,
                    preimage            // bytes32 _preimage
                ))
                .to.emit(tercSrc, "Transfer")
                .withArgs(
                    obridge.address,
                    lp.address,
                    token_amount_src
                )

                await expect(obridge.connect(lp).confirm(
                    lp.address,
                    user.address,
                    nativeTokenAddress,
                    token_amount_dst,
                    eth_amount,
                    hashlock,
                    timelock,
                    preimage
                ))
                .to.changeEtherBalance(user, token_amount_dst)
            })
        })
    })


})
