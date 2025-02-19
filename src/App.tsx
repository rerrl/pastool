import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import GenerateNewPass from "./components/GenerateNewPass";
import SearchPassStore from "./components/SearchPassStore";

interface SystemStatus {
  has_gpg: boolean;
  has_pass: boolean;
  has_pass_store: boolean;
  home_dir: string;
}

function App() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [screen, setScreen] = useState<"search" | "generate-new">("search");
  const [startupMessage, setStartupMessage] = useState("Initializing...");
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
    try {
      const res = (await invoke("initialize")) as SystemStatus;

      if (!res.has_pass_store) {
        setStartupMessage(
          "No password store found! Try `pass init` and try again."
        );
        return;
      }

      if (!res.has_pass) {
        setStartupMessage("No pass found! Install pass and try again.");
        return;
      }

      if (!res.has_gpg) {
        setStartupMessage("No gpg found! Install gpg and try again.");
        return;
      }

      setHomeDir(res.home_dir);

      await load_password_store();
      setIsInitialized(true);
    } catch (error) {
      console.error(error);
    }
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

      {!isInitialized ? (
        <h2 className="text-2xl font-bold my-4">{startupMessage}</h2>
      ) : (
        <>
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
        </>
      )}
    </main>
  );
}

export default App;
