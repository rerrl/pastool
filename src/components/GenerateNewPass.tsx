import { useState } from "react";
import Checkbox from "./Checkbox";

export default function GenerateNewPass() {
  // const [copyToClipboardOnCreate, setCopyToClipboardOnCreate] = useState(true);
  const [noSymbolOnPassword, setNoSymbolOnPassword] = useState(false);
  const [passwordLength, setPasswordLength] = useState(25);

  const onGenerateClick = () => {
    console.log(
      `Generating password with length ${passwordLength}, no symbols: ${noSymbolOnPassword}`
    );
  };

  return (
    <div className="flex flex-col items-center justify-center">
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
      {/* checkboxes */}
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

      <button
        onClick={onGenerateClick}
        className="bg-blue-500 w-full hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
      >
        Generate
      </button>
    </div>
  );
}
