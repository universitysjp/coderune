import { mountDesktop } from "../../app";
import { createApplication } from "../../src/application";

const dataDir = process.argv[2];

if (!dataDir) {
  throw new Error("Disposable application data directory required");
}

const runtime = await createApplication({
  dataDir,
  agents: {
    listAgents: () => [],
    getAgent: () => null,
    draft: async () => {
      throw new Error("Agent access is disabled in startup tests");
    },
  },
});

process.on("exit", () => runtime.close());

mountDesktop(runtime);
