import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function GenerateNewPass({
  passwordList,
  onFocusSearch,
}: {
  passwordList: string[];
  onFocusSearch: number;
}) {
  //   const [fullPasswordList, _] = useState<string[]>(passwordList);
  const [filteredPasswordList, setFilteredPasswordList] =
    useState<string[]>(passwordList);

  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Focus on the input when this effect is triggered
    if (searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [onFocusSearch]); // This effect runs when onFocusSearch changes

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
      <div className="fixed top-[140px] left-0 w-full z-10 flex items-center justify-center">
        <input
          ref={searchInputRef}
          className="my-4"
          type="text"
          placeholder="Search..."
          onChange={onSearchPathChange}
        />
      </div>

      <div className="flex-1 overflow-y-auto">
        {filteredPasswordList.map((entry) => (
          <div
            className="w-full wafer mb-2 p-2"
            onClick={() => {
              copyEncryptedPasswordToClipboard(entry);
            }}
            key={entry}
          >
            {entry}
          </div>
        ))}
      </div>
    </>
  );
}
