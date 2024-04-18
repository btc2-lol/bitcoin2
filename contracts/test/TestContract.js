const {
  time,
  loadFixture,
} = require("@nomicfoundation/hardhat-toolbox/network-helpers");
const { anyValue } = require("@nomicfoundation/hardhat-chai-matchers/withArgs");
const { expect } = require("chai");

describe("TestContract", function () {
  // We define a fixture to reuse the same setup in every test.
  // We use loadFixture to run this setup once, snapshot that state,
  // and reset Hardhat Network to that snapshot in every test.
  async function deployFixture() {
    const [owner, otherAccount] = await ethers.getSigners();

    const TestContract = await ethers.getContractFactory("TestContract");
    const test = await TestContract.deploy();

    return { test, owner, otherAccount };
  }

  describe("Storage", function () {
    it("should set a storage value", async function () {
      const { test } = await loadFixture(deployFixture);
      await test.set(1);
      expect(await test.get()).to.equal(1);
    });
  });
});
