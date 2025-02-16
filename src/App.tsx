import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PasswordStoreEntry {
  path: string;
  is_folder: boolean;
  is_encrypted: boolean;
}

function App() {
  const [passwordList, setPasswordList] = useState<PasswordStoreEntry[]>([]);

  const makePathRelativeToPasswordStore = (path: string) => {
    return path.split(".password-store/")[1];
  };

  async function load_password_store() {
    console.log("loading password store...");
    const result = (await invoke(
      "load_password_store"
    )) as PasswordStoreEntry[];
    setPasswordList(
      result.map((content) => {
        return {
          path: makePathRelativeToPasswordStore(content.path),
          is_folder: content.is_folder,
          is_encrypted: content.is_encrypted,
        };
      })
    );
  }

  useEffect(() => {
    load_password_store();
  }, []);

  return (
    <main className="flex flex-col items-center justify-center">
      <h1 className="text-3xl font-bold my-4">Password Store</h1>

      {passwordList.map((entry) => (
        <div className="row" key={entry.path}>
          {entry.path}
        </div>
      ))}
    </main>
  );
}

export default App;
