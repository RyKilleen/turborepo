/** @typedef {import('../types/backend').RuntimeBackend} RuntimeBackend */

/** @type {RuntimeBackend} */
let BACKEND;

(() => {
  /**
   * @param {ChunkPath} chunkPath
   */
  function loadChunk(chunkPath) {
    if (!chunkPath.endsWith(".js")) {
      // We only support loading JS chunks in Node.js.
      // This branch can be hit when trying to load a CSS chunk.
      return;
    }

    const path = require("path");
    const resolved = require.resolve(
      "./" + path.relative(path.dirname(RUNTIME_CHUNK_PATH), chunkPath)
    );
    delete require.cache[resolved];
    require(resolved);
  }

  BACKEND = {
    registerChunk(chunkPath, params) {
      if (params == null) {
        return;
      }

      if (
        params.runtimeModuleIds.length > 0 ||
        params.exportedCjsModuleId !== null
      ) {
        for (const otherChunkPath of params.otherChunks) {
          loadChunk(otherChunkPath);
        }
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

      if (params.exportedCjsModuleId !== null) {
        const mod = getOrInstantiateRuntimeModule(
          params.exportedCjsModuleId,
          chunkPath
        );
        module.exports = mod.exports;
      }
    },

    loadChunk(chunkPath, _source) {
      if (!chunkPath.endsWith(".js")) {
        // We only support loading JS chunks in Node.js.
        // This branch can be hit when trying to load a CSS chunk.
        return;
      }

      loadChunk(chunkPath);
    },
  };
})();
