// SPDX-License-Identifier: GPL-3.0-only

pragma solidity >=0.8.0 <0.9.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

contract OBridge {
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
        address dstAddress,
        uint64 bidId
    );
    event LogNewTransferIn(
        bytes32 transferId,
        address sender,
        address receiver,
        address token,
        uint256 amount,
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
        address _dstAddress,
        uint64 _bidId
    ) external {
        require( msg.sender == _sender, "require sender");

        bytes32 transferId = _transfer(_sender, _bridge, _token, _amount, _hashlock, _timelock);
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
            _bidId
        );
    }

    /**
     * @dev transfer sets up a new inbound transfer with hash time lock.
     */
    function transferIn(
        address _sender,
        address _dstAddress,
        address _token,
        uint256 _amount,
        bytes32 _hashlock,
        uint64 _timelock,
        uint64 _srcChainId,
        bytes32 _srcTransferId
    ) external {
        require( msg.sender == _sender, "require sender");

        bytes32 transferId = _transfer(_sender, _dstAddress, _token, _amount, _hashlock, _timelock);
        emit LogNewTransferIn(
            transferId,
            _sender,
            _dstAddress,
            _token,
            _amount,
            _hashlock,
            _timelock,
            _srcChainId,
            _srcTransferId
        );
    }

    /**
     * @dev confirm a transfer.
     *
     * @param _transferId Id of pending transfer.
     * @param _preimage key for the hashlock
     */
    function confirm(
        address _sender,
        address _receiver,
        address _token,
        uint256 _amount,
        bytes32 _hashlock,
        uint64 _timelock,
        bytes32 _preimage) external {

        _transferId = keccak256(abi.encodePacked(_sender, _receiver, _hashlock, _timelock, _token, _amount, block.chainid));
        TransferStatus memory t = transfers[_transferId];

        require(t == TransferStatus.Pending, "not pending transfer");
        require(hashlock == keccak256(abi.encodePacked(_preimage)), "incorrect preimage");

        transfers[_transferId] = TransferStatus.Confirmed;

        IERC20(_token).safeTransfer(_receiver, _amount);
        emit LogTransferConfirmed(_transferId, _preimage);
    }

    /**
     * @dev refund a transfer after timeout.
     *
     * @param _transferId Id of pending transfer.
     */
    function refund(
        address _sender,
        address _receiver,
        address _token,
        uint256 _amount,
        bytes32 _hashlock,
        uint64 _timelock) external {
        _transferId = keccak256(abi.encodePacked(_sender, _receiver, _hashlock, _timelock, _token, _amount, block.chainid));
        Transfer memory t = transfers[_transferId];

        require(t == TransferStatus.Pending, "not pending transfer");
        require(_timelock <= block.timestamp, "timelock not yet passed");

        transfers[_transferId] = TransferStatus.Refunded;

        IERC20(_token).safeTransfer(_sender, _amount);
        emit LogTransferRefunded(_transferId);
    }

    /**
     * @dev transfer sets up a new transfer with hash time lock.
     */
    function _transfer(
        address _sender,
        address _receiver,
        address _token,
        uint256 _amount,
        bytes32 _hashlock,
        uint64 _timelock
    ) private returns (bytes32 transferId) {
        require(_amount > 0, "invalid amount");
        require(_timelock > block.timestamp, "invalid timelock");

        transferId = keccak256(abi.encodePacked(_sender, _receiver, _hashlock, _timelock, _token, _amount, block.chainid));
        require(transfers[transferId] == TransferStatus.Null, "transfer exists");

        IERC20(_token).safeTransferFrom(_sender, address(this), _amount);

        transfers[transferId] = TransferStatus.Pending;
        return transferId;
    }
}
