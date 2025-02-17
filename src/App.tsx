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
      console.log(res.home_dir);

      await load_password_store();
      setIsInitialized(true);
    } catch (error) {
      console.error(error);
    }
  }

  useEffect(() => {
    initialize();
  }, []);

  return (
    <main className="flex flex-col items-center justify-center">
      <h1 className="text-3xl font-bold my-4">Password Store</h1>
      {isInitialized && (
        <>
          <div className="flex pb-6 border-b-2 w-full items-center justify-center my-4 space-x-5">
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
          {/* <div className="w-full border-2 border-b-2"></div> */}
        </>
      )}
      {!isInitialized ? (
        <h2 className="text-2xl font-bold my-4">{startupMessage}</h2>
      ) : (
        <>
          {screen === "search" ? (
            <SearchPassStore passwordList={fullPasswordList} />
          ) : (
            <GenerateNewPass />
          )}
        </>
      )}
    </main>
  );
}

export default App;
