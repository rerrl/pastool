export default function Checkbox({
  checked,
  onChange,
  id,
  value,
}: {
  checked: boolean;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  id: string;
  value: string;
}) {
  return (
    <div className="flex items-center space-x-2">
      <input
        type="checkbox"
        name={id}
        id={id}
        value={value}
        checked={checked}
        onChange={onChange}
      />
      <label htmlFor={id}>{value}</label>
    </div>
  );
}
