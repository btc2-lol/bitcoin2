// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

contract TestContract {
    uint256 private storedData;

    function set(uint256 value) public {
        storedData = value;
    }

    function get() public view returns (uint256) {
        return storedData;
    }
}
