import { LocalCodingAgentGateway } from "@coderune/agents";
import type { LocalCodingAgentGatewayOptions } from "@coderune/agents";
import {
  LocalDocumentExtractor,
  LocalRepositoryCollector,
} from "@coderune/collectors";
import { ApplicationOperations } from "@coderune/core";
import type {
  Clock,
  CodingAgentGateway,
  ContextStore,
} from "@coderune/core";
import {
  resolveDatabasePath,
  SqliteContextStore,
} from "@coderune/storage";

export interface CreateApplicationOptions {
  dataDir: string;
  clock?: Clock;
  store?: ContextStore;
  agents?: CodingAgentGateway;
  agentOptions?: LocalCodingAgentGatewayOptions;
}

export interface ApplicationRuntime {
  operations: ApplicationOperations;
  store: ContextStore;
  close: () => void;
}

export const createApplication = async (
  options: CreateApplicationOptions
): Promise<ApplicationRuntime> => {
  const store =
    options.store ??
    new SqliteContextStore({
      databasePath: resolveDatabasePath(options.dataDir),
    });

  await store.initialize();

  const operations = new ApplicationOperations({
    store,
    documents: new LocalDocumentExtractor(),
    repository: new LocalRepositoryCollector(),
    agents:
      options.agents ?? new LocalCodingAgentGateway(options.agentOptions ?? {}),
    clock: options.clock,
  });

  return {
    operations,
    store,
    close: () => store.close(),
  };
};
