import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SystemStatus {
  has_gpg: boolean;
  has_pass: boolean;
  has_pass_store: boolean;
}

function App() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [startupMessage, setStartupMessage] = useState("Initializing...");
  const [fullPasswordList, setFullPasswordList] = useState<string[]>([]);
  const [filteredPasswordList, setFilteredPasswordList] = useState<string[]>(
    []
  );

  const makePathRelativeToPasswordStore = (path: string) => {
    return path.split(".password-store/")[1];
  };

  const onSearchPathChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.value.length === 0) {
      setFilteredPasswordList(fullPasswordList);
      return;
    }

    const searchStrings = e.target.value.split(" ");
    setFilteredPasswordList(
      fullPasswordList.filter((entry) => {
        return searchStrings.every((searchString) => {
          return entry.includes(searchString);
        });
      })
    );
  };

  async function copyEncryptedPasswordToClipboard(relativePath: string) {
    try {
      const result = (await invoke("copy_encrypted_password_to_clipboard", {
        relativePath: relativePath.replace(".gpg", ""),
      })) as string;

      alert(result);
    } catch (error) {
      console.error(error);
    }
  }

  async function load_password_store() {
    const paths = (await invoke("load_password_store")) as string[];

    const initialPasswordList = paths.map((path) =>
      makePathRelativeToPasswordStore(path)
    );
    setFullPasswordList(initialPasswordList);
    setFilteredPasswordList(initialPasswordList);
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
      {!isInitialized ? (
        <h2 className="text-2xl font-bold my-4">{startupMessage}</h2>
      ) : (
        <>
          <input
            className="my-4"
            type="text"
            placeholder="Search..."
            onChange={onSearchPathChange}
          />

          {filteredPasswordList.map((entry) => (
            <div
              className="py-1 w-full bg-black rounded-lg mb-2 px-2 hover:bg-gray-800 cursor-pointer"
              onClick={() => {
                console.log("clicked on " + entry);
                copyEncryptedPasswordToClipboard(entry);
              }}
              key={entry}
            >
              {entry}
            </div>
          ))}
        </>
      )}
    </main>
  );
}

export default App;
