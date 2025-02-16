import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface PasswordStoreEntry {
  path: string;
  is_folder: boolean;
  is_encrypted: boolean;
}

function App() {
  const [passwordList, setPasswordList] = useState<PasswordStoreEntry[]>([]);

  const makePathRelativeToPasswordStore = (path: string) => {
    return path.split(".password-store/")[1];
  }

  async function load_password_store() {
    console.log("loading password store...");
    const result = await invoke("load_password_store") as PasswordStoreEntry[];
    setPasswordList(result.map(content => {
      return {
        path: makePathRelativeToPasswordStore(content.path),
        is_folder: content.is_folder,
        is_encrypted: content.is_encrypted,
      }
    }));
  }

  useEffect(() => {
    load_password_store();
  }, []);

  return (
    <main className="container">
      <h1>Password Store</h1>
      
        {passwordList.map((entry) => (
          <div className="row" key={entry.path}>
            {entry.path}
          </div>
        ))}
      
    </main>
  );
}

export default App;
