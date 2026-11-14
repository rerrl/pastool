import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import GenerateNewPass from "./components/GenerateNewPass";
import OnboardingWizard from "./components/OnboardingWizard";
import SearchPassStore from "./components/SearchPassStore";
import type { GpgKey, SystemStatus } from "./types";

function App() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [screen, setScreen] = useState<"search" | "generate-new">("search");
  const [startupMessage, setStartupMessage] = useState("Initializing...");
  const [initError, setInitError] = useState("");
  const [fullPasswordList, setFullPasswordList] = useState<string[]>([]);
  const [homeDir, setHomeDir] = useState("");
  const [triggerFocusSearch, setTriggerFocusSearch] = useState(0);

  const makePathRelativeToPasswordStore = (path: string) => {
    return path.split(".password-store/")[1];
  };

  async function load_password_store() {
    const paths = (await invoke("load_password_store")) as string[];

    const initialPasswordList = paths.map((path) =>
      makePathRelativeToPasswordStore(path)
    );
    setFullPasswordList(initialPasswordList);
  }

  async function initialize() {
    setInitError("");
    setStartupMessage("Initializing...");
    try {
      const res = (await invoke("initialize")) as SystemStatus;
      setStatus(res);

      // Everything present *and* the user already has a key to unlock the
      // store: skip onboarding entirely.
      if (res.has_gpg && res.has_pass && res.has_pass_store) {
        const keys = (await invoke("list_secret_keys")) as GpgKey[];
        if (keys.length > 0) {
          setHomeDir(res.home_dir);
          await load_password_store();
          setIsInitialized(true);
          return;
        }
      }

      // Something is missing -- the wizard collects the rest.
      setIsInitialized(false);
    } catch (error) {
      console.error(error);
      setInitError(
        typeof error === "string"
          ? error
          : "Something went wrong while starting up."
      );
    }
  }

  async function finishOnboarding() {
    if (status) {
      setHomeDir(status.home_dir);
    }
    await load_password_store();
    setIsInitialized(true);
  }

  useEffect(() => {
    initialize();
  }, []);

  useEffect(() => {
    if (screen === "search") {
      setTriggerFocusSearch(triggerFocusSearch + 1);
    }
  }, [screen]);

  return (
    <main className="flex flex-col items-center justify-center cake-bg h-screen">
      <div className="fixed top-0 left-0 w-full z-10">
        <h1 className="text-[64px] mt-8 mb-5 title">Pastool</h1>
        {isInitialized && (
          <>
            <div className="flex pb-6 w-full items-center justify-center my-4 space-x-5 text-[#664316]">
              {/* radio buttons */}
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  name="screen"
                  id="search"
                  value="search"
                  checked={screen === "search"}
                  onChange={() => setScreen("search")}
                />
                <label htmlFor="search">Search</label>
              </div>
              <div className="flex items-center space-x-2">
                <input
                  type="radio"
                  name="screen"
                  id="generate-new"
                  value="generate-new"
                  checked={screen === "generate-new"}
                  onChange={() => setScreen("generate-new")}
                />
                <label htmlFor="generate-new">Generate New</label>
              </div>
            </div>
          </>
        )}
      </div>

      {isInitialized ? (
        <div
          className={`flex-1 overflow-y-auto ${
            screen === "search" ? "mt-[220px]" : "mt-[150px]"
          } w-3/4 max-w-md`}
        >
          {screen === "search" ? (
            <SearchPassStore
              passwordList={fullPasswordList}
              onFocusSearch={triggerFocusSearch}
            />
          ) : (
            <GenerateNewPass homeDir={homeDir} />
          )}
        </div>
      ) : (
        <div className="flex-1 w-3/4 max-w-lg overflow-y-auto mt-[130px] mb-6">
          {status === null ? (
            initError ? (
              <div className="flex flex-col items-center text-center space-y-4">
                <h2 className="text-2xl font-bold my-4">
                  We couldn't start up
                </h2>
                <div className="text-red-600 text-sm max-w-md">{initError}</div>
                <button className="wafer" onClick={initialize}>
                  Try again
                </button>
              </div>
            ) : (
              <h2 className="text-2xl font-bold my-4 text-center">
                {startupMessage}
              </h2>
            )
          ) : (
            <OnboardingWizard
              status={status}
              onRecheck={initialize}
              onComplete={finishOnboarding}
            />
          )}
        </div>
      )}
    </main>
  );
}

export default App;
