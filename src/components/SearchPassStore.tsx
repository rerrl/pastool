import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function GenerateNewPass({
  passwordList,
}: {
  passwordList: string[];
}) {
  //   const [fullPasswordList, _] = useState<string[]>(passwordList);
  const [filteredPasswordList, setFilteredPasswordList] =
    useState<string[]>(passwordList);

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

  const onSearchPathChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.value.length === 0) {
      setFilteredPasswordList(passwordList);
      return;
    }

    const searchStrings = e.target.value.split(" ");
    setFilteredPasswordList(
        passwordList.filter((entry) => {
        return searchStrings.every((searchString) => {
          return entry.includes(searchString);
        });
      })
    );
  };

  return (
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
            copyEncryptedPasswordToClipboard(entry);
          }}
          key={entry}
        >
          {entry}
        </div>
      ))}
    </>
  );
}
