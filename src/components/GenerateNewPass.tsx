import { useState } from "react";
import Checkbox from "./Checkbox";

export default function GenerateNewPass() {
  const [copyToClipboardOnCreate, setCopyToClipboardOnCreate] = useState(true);
  const [noSymbolOnPassword, setNoSymbolOnPassword] = useState(false);

  return (
    <div>
      <div>Password Path</div>
      <input
        className="my-4"
        type="text"
        placeholder="Password Path"
        value="~/.passwords/password.gpg"
      />

      <div>Password Length</div>
      <input
        className="my-4"
        type="number"
        placeholder="Password Length"
        value="16"
      />
      {/* checkboxes */}
      <div className="flex items-center justify-center my-4 space-x-5">
        <Checkbox
          checked={copyToClipboardOnCreate}
          onChange={(e) => {
            setCopyToClipboardOnCreate(e.target.checked);
          }}
          id="copy-to-clipboard"
          value="Copy to Clipboard"
        />
        <Checkbox
          checked={noSymbolOnPassword}
          onChange={(e) => {
            setNoSymbolOnPassword(e.target.checked);
          }}
          id="no-symbol-on-password"
          value="No Symbol on Password"
        />
      </div>
    </div>
  );
}
