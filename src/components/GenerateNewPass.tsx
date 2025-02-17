import { useState } from "react";
import Checkbox from "./Checkbox";
import { invoke } from "@tauri-apps/api/core";

export default function GenerateNewPass() {
  // const [copyToClipboardOnCreate, setCopyToClipboardOnCreate] = useState(true);
  const [noSymbolOnPassword, setNoSymbolOnPassword] = useState(false);
  const [passwordLength, setPasswordLength] = useState(25);
  const [saveFolder, setSaveFolder] = useState("~/.password-store");
  const [passwordName, setPasswordName] = useState("");

  const onGenerateClick = () => {
    console.log(
      `Generating password with length ${passwordLength}, no symbols: ${noSymbolOnPassword}`
    );
  };

  const onSelectFolderClick = async () => {
    console.log("Selecting folder");
    try {
      const selectedFolder = (await invoke("open_dialog")) as string;
      setSaveFolder(selectedFolder);
    } catch (error) {
      if (error !== "No folder selected") {
        console.error(error);
      }
    }
  };

  return (
    <>
      <div className="flex flex-col items-center justify-center space-y-4 w-full">
        {/* save folder */}
        <div>
          <div>Save Folder:</div>
          <button
            onClick={onSelectFolderClick}
            className="bg-blue-500 w-full hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
          >
            {saveFolder}
          </button>
        </div>

        {/* password name */}
        <div>
          <div>Password Name:</div>
          <input
            type="text"
            value={passwordName}
            onChange={(e) => {
              // ignore spaces
              if (e.target.value.includes(" ")) {
                return;
              }
              setPasswordName(e.target.value);
            }}
          />
        </div>

        {/* password length */}
        <div className="flex items-center justify-center space-x-5">
          <div>Password Length</div>
          <input
            type="number"
            placeholder="Password Length"
            onChange={(e) =>
              setPasswordLength(parseInt(e.target.value as string))
            }
            value={passwordLength}
            min={1}
            max={100}
          />
        </div>

        {/* no symbols */}
        <div className="flex items-center justify-center space-x-5">
          <Checkbox
            checked={noSymbolOnPassword}
            onChange={(e) => {
              setNoSymbolOnPassword(e.target.checked);
            }}
            id="no-symbols"
            value="No Symbols"
          />
        </div>

        <div className="w-full border border-b-2" />

        {/* final path */}
        <div>
          <span className="text-blue-300">
            ~/.password-store{saveFolder.split(".password-store")[1]}
          </span>
          <span className="text-green-600">
            /{passwordName.replace(".gpg", "")}
          </span>
          <span className="text-red-600">.gpg</span>
        </div>

        {/* generate pass button */}
        <button
          onClick={onGenerateClick}
          className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
        >
          Generate
        </button>
      </div>
    </>
  );
}
