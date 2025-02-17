import { useEffect, useState } from "react";
import Checkbox from "./Checkbox";

export default function GenerateNewPass() {
  // const [copyToClipboardOnCreate, setCopyToClipboardOnCreate] = useState(true);
  const [noSymbolOnPassword, setNoSymbolOnPassword] = useState(false);
  const [passwordLength, setPasswordLength] = useState(25);

  useEffect(() => {
    console.log(passwordLength);
  }, [passwordLength]);

  return (
    <div>
      <div className="flex items-center justify-center my-4 space-x-5">
        <div>Password Length</div>
        <input
          className="my-4"
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
      <div className="flex items-center justify-center my-4 space-x-5">
        <Checkbox
          checked={noSymbolOnPassword}
          onChange={(e) => {
            setNoSymbolOnPassword(e.target.checked);
          }}
          id="no-symbols"
          value="No Symbols"
        />
      </div>
    </div>
  );
}
