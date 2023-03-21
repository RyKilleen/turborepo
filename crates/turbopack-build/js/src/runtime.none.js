/** @typedef {import('../types/backend').RuntimeBackend} RuntimeBackend */

/** @type {RuntimeBackend} */
const BACKEND = {
  registerChunk(chunkPath, params) {
    if (params == null) {
      return;
    }

    for (const moduleId of params.runtimeModuleIds) {
      try {
        getOrInstantiateRuntimeModule(moduleId, chunkPath);
      } catch (err) {
        console.error(
          `The following error occurred while evaluating runtime entries of ${chunkPath}:`
        );
        console.error(err);
        return;
      }
    }
  },

  loadChunk(chunkPath, source) {
    throw new Error("chunk loading is not supported");
  },

  registerChunk(chunkPath, params) {
    throw new Error("chunk loading is not supported");
  },
};
