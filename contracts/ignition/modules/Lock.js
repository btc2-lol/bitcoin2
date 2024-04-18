const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

const JAN_1ST_2030 = 1893456000;
const ONE_GWEI = 1_000_000_000n;

module.exports = buildModule("TestContractModule", (m) => {
  const unTestContractTime = m.getParameter("unTestContractTime", JAN_1ST_2030);
  const TestContractedAmount = m.getParameter("TestContractedAmount", ONE_GWEI);

  const TestContract = m.contract("TestContract", [unTestContractTime], {
    value: TestContractedAmount,
  });

  return { TestContract };
});
