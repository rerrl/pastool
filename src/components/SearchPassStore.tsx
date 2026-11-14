import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function SearchPassStore({
  passwordList,
  onFocusSearch,
}: {
  passwordList: string[];
  onFocusSearch: number;
}) {
  const [filteredPasswordList, setFilteredPasswordList] =
    useState<string[]>(passwordList);
  const [copiedEntry, setCopiedEntry] = useState<string | null>(null);
  const [copyError, setCopyError] = useState("");

  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Focus on the input when this effect is triggered
    if (searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [onFocusSearch]); // This effect runs when onFocusSearch changes

  useEffect(() => {
    if (copiedEntry === null) return;
    const timeout = setTimeout(() => setCopiedEntry(null), 2000);
    return () => clearTimeout(timeout);
  }, [copiedEntry]);

  async function copyEncryptedPasswordToClipboard(relativePath: string) {
    setCopyError("");
    try {
      await invoke("copy_encrypted_password_to_clipboard", {
        relativePath: relativePath.replace(".gpg", ""),
      });
      setCopiedEntry(relativePath);
    } catch (error) {
      console.error(error);
      setCopyError(typeof error === "string" ? error : "Failed to copy.");
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

      {copyError && (
        <div className="text-red-600 text-sm text-center mb-2">{copyError}</div>
      )}

      <div className="flex-1 overflow-y-auto">
        {filteredPasswordList.map((entry) => (
          <div
            className="w-full wafer mb-2 p-2 flex items-center justify-between"
            onClick={() => {
              copyEncryptedPasswordToClipboard(entry);
            }}
            key={entry}
          >
            <span>{entry}</span>
            {copiedEntry === entry && (
              <span className="ml-2 text-sm opacity-70">Copied!</span>
            )}
          </div>
        ))}
      </div>
    </>
  );
}
