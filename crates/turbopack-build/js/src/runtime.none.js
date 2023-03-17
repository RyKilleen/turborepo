/** @typedef {import('../types/backend').RuntimeBackend} RuntimeBackend */

/** @type {RuntimeBackend} */
const BACKEND = {
  loadChunk(chunkPath, source) {
    throw new Error("chunk loading is not supported");
  },

  registerChunk(chunkPath, params) {
    throw new Error("chunk loading is not supported");
  },
};
