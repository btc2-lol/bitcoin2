const {
  time,
  loadFixture,
} = require("@nomicfoundation/hardhat-toolbox/network-helpers");
const { anyValue } = require("@nomicfoundation/hardhat-chai-matchers/withArgs");
const { expect } = require("chai");

describe("SystemContract", function () {
  async function deployFixture() {
    const [owner, otherAccount] = await ethers.getSigners();

    const SystemContract = await ethers.getContractFactory("System");
    const system = await SystemContract.deploy();

    return { system, owner, otherAccount };
  }
});
