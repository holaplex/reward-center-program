// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
  idlGenerator: 'anchor',
  programName: 'reward_center',
  programId: 'RwDDvPp7ta9qqUwxbBfShsNreBaSsKvFcHzMxfBC3Ki',
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
};
