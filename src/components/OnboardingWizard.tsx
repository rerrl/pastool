import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GpgKey, SystemStatus } from "../types";

type Step = "install" | "key" | "init" | "done";

const INSTALL_COMMANDS: Record<string, string> = {
  apt: "sudo apt update && sudo apt install -y gnupg pass",
  dnf: "sudo dnf install -y gnupg2 pass",
  pacman: "sudo pacman -S --needed gnupg pass",
};

const TOTAL_STEPS = 3;

function installCommandFor(pkgManager: string): string {
  return (
    INSTALL_COMMANDS[pkgManager] ??
    "Use your system's package manager to install the 'gpg' and 'pass' packages."
  );
}

function errorText(error: unknown, fallback: string): string {
  return typeof error === "string" && error.length > 0 ? error : fallback;
}

export default function OnboardingWizard({
  status,
  onRecheck,
  onComplete,
}: {
  status: SystemStatus;
  onRecheck: () => Promise<void>;
  onComplete: () => Promise<void>;
}) {
  const [step, setStep] = useState<Step>("install");
  const [keys, setKeys] = useState<GpgKey[]>([]);
  const [selectedKeyId, setSelectedKeyId] = useState("");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const toolsMissing =
    !status.has_gpg || !status.has_pass
      ? [!status.has_gpg ? "gpg" : null, !status.has_pass ? "pass" : null]
          .filter((t): t is string => t !== null)
          .join(" and ")
      : "";

  // Recompute which step the user should be on whenever system status changes
  // (e.g. after they click "Re-check" following an install).
  useEffect(() => {
    if (!status.has_gpg || !status.has_pass) {
      setStep("install");
      return;
    }

    let cancelled = false;
    (async () => {
      try {
        const found = await invoke<GpgKey[]>("list_secret_keys");
        if (cancelled) return;
        setError("");
        setKeys(found);
        setSelectedKeyId((current) => current || found[0]?.id || "");

        if (found.length === 0) {
          setStep("key");
        } else if (!status.has_pass_store) {
          setStep("init");
        } else {
          setStep("done");
        }
      } catch (e) {
        if (cancelled) return;
        setError(errorText(e, "Could not read your GPG keys. Please try again."));
        setStep("key");
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [status]);

  const handleRecheck = useCallback(async () => {
    setBusy(true);
    setError("");
    try {
      await onRecheck();
    } finally {
      setBusy(false);
    }
  }, [onRecheck]);

  const handleGenerate = async () => {
    setBusy(true);
    setError("");
    try {
      const key = await invoke<GpgKey>("generate_gpg_key", { name, email });
      setKeys((current) => [
        ...current.filter((k) => k.id !== key.id),
        key,
      ]);
      setSelectedKeyId(key.id);
      setStep(status.has_pass_store ? "done" : "init");
    } catch (e) {
      setError(errorText(e, "Could not create your key. Please try again."));
    } finally {
      setBusy(false);
    }
  };

  const handleInit = async () => {
    setBusy(true);
    setError("");
    try {
      await invoke("init_pass_store", { keyId: selectedKeyId });
      setStep("done");
    } catch (e) {
      setError(errorText(e, "Could not set up your password store. Please try again."));
    } finally {
      setBusy(false);
    }
  };

  const selectedKey = keys.find((k) => k.id === selectedKeyId);

  if (step === "done") {
    return (
      <div className="flex flex-col items-center text-center space-y-5 py-4">
        <h2 className="text-3xl font-bold">All set! 🧁</h2>
        <p className="max-w-md">
          Your password store is ready to go. Let's find your passwords.
        </p>
        <button className="wafer" onClick={onComplete}>
          Start using Pastool
        </button>
      </div>
    );
  }

  const stepNumber = step === "install" ? 1 : step === "key" ? 2 : 3;

  return (
    <div className="flex flex-col items-center text-center space-y-4 py-4 w-full">
      <div className="text-sm opacity-70">
        Setup — step {stepNumber} of {TOTAL_STEPS}
      </div>

      {step === "install" && (
        <>
          <h2 className="text-2xl font-bold">Let's get you set up</h2>
          <p className="max-w-md">
            Pastool needs two free tools to keep your passwords safe, and we
            couldn't find {toolsMissing} on this computer.
          </p>
          <p className="max-w-md">
            Open a terminal and paste this command to install them:
          </p>
          <code className="block max-w-md w-full text-left bg-[#664316]/85 text-white rounded-lg p-3 text-sm break-words">
            {installCommandFor(status.pkg_manager)}
          </code>
          <p className="text-sm max-w-md opacity-80">
            Already installed them? Give us a moment and try again.
          </p>
          {error && (
            <div className="text-red-600 text-sm max-w-md">{error}</div>
          )}
          <button className="wafer" onClick={handleRecheck} disabled={busy}>
            {busy ? "Checking…" : "Re-check"}
          </button>
        </>
      )}

      {step === "key" && (
        <>
          <h2 className="text-2xl font-bold">Create your secret key</h2>
          <p className="max-w-md">
            A GPG key is a pair of secret codes that locks and unlocks your
            passwords. You keep the private half on this computer, and it's the
            only thing that can read your password store. We'll make one for you
            now — it only takes a few seconds.
          </p>
          <div className="flex flex-col items-center space-y-1 w-full max-w-md">
            <label htmlFor="gpg-name" className="self-start">
              Your name
            </label>
            <input
              id="gpg-name"
              className="w-full"
              type="text"
              placeholder="Ada Lovelace"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={busy}
            />
          </div>
          <div className="flex flex-col items-center space-y-1 w-full max-w-md">
            <label htmlFor="gpg-email" className="self-start">
              Your email
            </label>
            <input
              id="gpg-email"
              className="w-full"
              type="email"
              placeholder="ada@example.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={busy}
            />
          </div>
          <p className="text-sm max-w-md opacity-80">
            These are just a label so you can tell your keys apart later —
            nothing is sent anywhere.
          </p>
          {error && (
            <div className="text-red-600 text-sm max-w-md">{error}</div>
          )}
          <button
            className="wafer"
            onClick={handleGenerate}
            disabled={busy || name.trim().length === 0 || email.trim().length === 0}
          >
            {busy ? "Creating your key…" : "Create my key"}
          </button>
        </>
      )}

      {step === "init" && (
        <>
          <h2 className="text-2xl font-bold">Set up your password store</h2>
          <p className="max-w-md">
            Almost done! We'll create your encrypted password folder and tell it
            to use your key.
          </p>
          {keys.length > 1 ? (
            <div className="flex flex-col items-center space-y-1 w-full max-w-md">
              <label htmlFor="gpg-key" className="self-start">
                Which key should lock your passwords?
              </label>
              <select
                id="gpg-key"
                className="wafer w-full"
                value={selectedKeyId}
                onChange={(e) => setSelectedKeyId(e.target.value)}
                disabled={busy}
              >
                {keys.map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.uid || key.id}
                  </option>
                ))}
              </select>
            </div>
          ) : (
            <p className="text-sm max-w-md opacity-80">
              Using key: {selectedKey?.uid || selectedKeyId}
            </p>
          )}
          {error && (
            <div className="text-red-600 text-sm max-w-md">{error}</div>
          )}
          <button
            className="wafer"
            onClick={handleInit}
            disabled={busy || selectedKeyId.length === 0}
          >
            {busy ? "Setting up…" : "Set up password store"}
          </button>
        </>
      )}
    </div>
  );
}
