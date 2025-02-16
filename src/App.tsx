import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
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
      await invoke("copy_encrypted_password_to_clipboard", {
        relativePath: relativePath.replace(".gpg", ""),
      });

      alert("Copied to clipboard!");
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

  useEffect(() => {
    load_password_store();
  }, []);

  return (
    <main className="flex flex-col items-center justify-center">
      <h1 className="text-3xl font-bold my-4">Password Store</h1>
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
    </main>
  );
}

export default App;
