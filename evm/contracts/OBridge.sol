// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

abstract contract Context {
    function _msgSender() internal view virtual returns (address) {
        return msg.sender;
    }

    function _msgData() internal view virtual returns (bytes calldata) {
        this; // silence state mutability warning without generating bytecode - see https://github.com/ethereum/solidity/issues/2691
        return msg.data;
    }
}

abstract contract Ownable is Context {
    address public owner;
    address public nextOwner;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    /**
     * @dev Initializes the contract setting the deployer as the initial owner.
     */
    constructor() {
        _setOwner(_msgSender());
    }

    /**
     * @dev Throws if called by any account other than the owner.
     */
    modifier onlyOwner() {
        require(owner == _msgSender(), "Ownable: caller is not the owner");
        _;
    }

    /**
     * @dev Leaves the contract without owner. It will not be possible to call
     * `onlyOwner` functions anymore. Can only be called by the current owner.
     *
     * NOTE: Renouncing ownership will leave the contract without an owner,
     * thereby removing any functionality that is only available to the owner.
     */
    function renounceOwnership() public virtual onlyOwner {
        _setOwner(address(0));
    }

    /**
    * @dev Allows the current owner to transfer control of the contract to a newOwner.
    * @param _newOwner The address to transfer ownership to.
    */
    function transferOwnership(address _newOwner) public onlyOwner {
        require(_newOwner != address(0), "Address should not be 0x");
        nextOwner = _newOwner;
    }
    
    function approveOwnership() public{
        require(nextOwner == msg.sender);
        owner = nextOwner;
    }

    function _setOwner(address newOwner) private {
        address oldOwner = owner;
        owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}

contract BridgeFee is Ownable{
    uint256 public basisPointsRate = 0;
    mapping(address => uint256) public maximumFee;
    address public tollAddress;

    /**
     * @dev Initializes the contract setting the deployer as the initial owner.
     */
    constructor() {
        tollAddress = _msgSender();
    }

    function setBasisPointsRate(uint256 rate) external onlyOwner{
        basisPointsRate = rate;
    }

    function setMaximumFee(address token, uint256 fee) external onlyOwner{
        maximumFee[token] = fee;
    }

    function setTollAddress(address toll) external onlyOwner {
        tollAddress = toll;
    }

    function calcFee(address token, uint256 value) view internal returns (uint256) {
        uint256 fee = value * basisPointsRate / 10000;

        uint256 maxFee = maximumFee[token];
        if (maxFee > 0 && fee > maxFee){
            fee = maxFee;
        }

        return fee;
    }
}

contract OBridge is BridgeFee{
    using SafeERC20 for IERC20;

    enum TransferStatus {
        Null,
        Pending,
        Confirmed,
        Refunded
    }

    mapping(bytes32 => TransferStatus) public transfers;

    event LogNewTransferOut(
        bytes32 transferId,
        address sender,
        address receiver,
        address token,
        uint256 amount,
        bytes32 hashlock, // hash of the preimage
        uint64 timelock, // UNIX timestamp seconds - locked UNTIL this time
        uint64 dstChainId,
        uint256 dstAddress,
        uint64 bidId,
        uint256 tokenDst,
        uint256 amountDst
    );
    event LogNewTransferIn(
        bytes32 transferId,
        address sender,
        address receiver,
        address token,
        uint256 token_amount,
        uint256 eth_amount,
        bytes32 hashlock, // hash of the preimage
        uint64 timelock, // UNIX timestamp seconds - locked UNTIL this time
        uint64 srcChainId,
        bytes32 srcTransferId // outbound transferId at src chain
    );
    event LogTransferConfirmed(bytes32 transferId, bytes32 preimage);
    event LogTransferRefunded(bytes32 transferId);

    /**
     * @dev transfer sets up a new outbound transfer with hash time lock.
     */
    function transferOut(
        address _sender,
        address _bridge,
        address _token,
        uint256 _amount,
        bytes32 _hashlock,
        uint64 _timelock,
        uint64 _dstChainId,
        uint256 _dstAddress,
        uint64 _bidId,
        uint256 _tokenDst,
        uint256 _amountDst
    ) external payable {
        require( msg.sender == _sender, "require sender");
        
        bytes32 transferId = _transfer(_sender, _bridge, _token, _amount, 0, _hashlock, _timelock);
        emit LogNewTransferOut(
            transferId,
            _sender,
            _bridge,
            _token,
            _amount,
            _hashlock,
            _timelock,
            _dstChainId,
            _dstAddress,
            _bidId,
            _tokenDst,
            _amountDst
        );
    }

    /**
     * @dev transfer sets up a new inbound transfer with hash time lock.
     */
    function transferIn(
        address _sender,
        address _dstAddress,
        address _token,
        uint256 _token_amount,
        uint256 _eth_amount,
        bytes32 _hashlock,
        uint64 _timelock,
        uint64 _srcChainId,
        bytes32 _srcTransferId
    ) external payable  {
        require( msg.sender == _sender, "require sender");

        bytes32 transferId = _transfer(_sender, _dstAddress, _token, _token_amount, _eth_amount, _hashlock, _timelock);
        emit LogNewTransferIn(
            transferId,
            _sender,
            _dstAddress,
            _token,
            _token_amount,
            _eth_amount,
            _hashlock,
            _timelock,
            _srcChainId,
            _srcTransferId
        );
    }

    
    function confirm(
        address _sender,
        address _receiver,
        address _token,
        uint256 _token_amount,
        uint256 _eth_amount,
        bytes32 _hashlock,
        uint64 _timelock,
        bytes32 _preimage) external {

        bytes32 _transferId = keccak256(abi.encodePacked(_sender, _receiver, _hashlock, _timelock, _token, _token_amount, _eth_amount, block.chainid));
        TransferStatus t = transfers[_transferId];

        require(t == TransferStatus.Pending, "not pending transfer");
        require(_hashlock == keccak256(abi.encodePacked(_preimage)), "incorrect preimage");

        transfers[_transferId] = TransferStatus.Confirmed;

        if( _token == address(0) ) {
            uint256 fee = calcFee(_token, _token_amount);
            uint256 sendAmount = _token_amount - fee;

            (bool sent, bytes memory data) = _receiver.call{value: sendAmount}("");
            require(sent, "Failed to send Ether");

            (sent, data) = tollAddress.call{value: fee}("");
            require(sent, "Failed to send Ether");
        } else {
            uint256 fee = calcFee(_token, _token_amount);
            uint256 sendAmount = _token_amount - fee;

            IERC20(_token).safeTransfer(_receiver, sendAmount);
            IERC20(_token).safeTransfer(tollAddress, fee);
            if( _eth_amount > 0 ) {

                fee = calcFee(address(0), _eth_amount);
                sendAmount = _eth_amount - fee;

                (bool sent, bytes memory data) = _receiver.call{value: sendAmount}("");
                require(sent, "Failed to send Ether");

                (sent, data) = tollAddress.call{value: fee}("");
                require(sent, "Failed to send Ether");
            }
        } 
        emit LogTransferConfirmed(_transferId, _preimage);
    }

   
    function refund(
        address _sender,
        address _receiver,
        address _token,
        uint256 _token_amount,
        uint256 _eth_amount,
        bytes32 _hashlock,
        uint64 _timelock) external {
        bytes32 _transferId = keccak256(abi.encodePacked(_sender, _receiver, _hashlock, _timelock, _token, _token_amount, _eth_amount, block.chainid));
        TransferStatus t = transfers[_transferId];

        require(t == TransferStatus.Pending, "not pending transfer");
        require(_timelock <= block.timestamp, "timelock not yet passed");

        transfers[_transferId] = TransferStatus.Refunded;

        if( _token == address(0) ) {
            (bool sent, ) = _sender.call{value: _token_amount}("");
            require(sent, "Failed to send Ether");
        } else {
            IERC20(_token).safeTransfer(_sender, _token_amount);
            if( _eth_amount > 0 ) {
                (bool sent, ) = _sender.call{value: _eth_amount}("");
                require(sent, "Failed to send Ether");
            }
        }
        emit LogTransferRefunded(_transferId);
    }

    
    function _transfer(
        address _sender,
        address _receiver,
        address _token,
        uint256 _token_amount,
        uint256 _eth_amount,
        bytes32 _hashlock,
        uint64 _timelock
    ) private returns (bytes32 transferId) {
        require(_token_amount > 0, "invalid amount");
        require(_timelock > block.timestamp, "invalid timelock");

        transferId = keccak256(abi.encodePacked(_sender, _receiver, _hashlock, _timelock, _token, _token_amount, _eth_amount, block.chainid));
        require(transfers[transferId] == TransferStatus.Null, "transfer exists");


        if( _token == address(0) ) {
            require(_eth_amount == 0, "Eth Amount should zero");
            require(_token_amount == msg.value, "Eth Amount mismatch");
        } else {
            require(_eth_amount == msg.value, "Eth Amount mismatch");
            IERC20(_token).safeTransferFrom(_sender, address(this), _token_amount);
        }
         

        transfers[transferId] = TransferStatus.Pending;
        return transferId;
    }

    receive() external payable {}
}