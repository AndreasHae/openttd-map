export interface FilePickerProps {
  onFileChanged: (file?: File) => void;
}

export function FilePicker({ onFileChanged }: FilePickerProps) {
  return <input type="file" onChange={(event) => onFileChanged(event.target.files?.[0])} />;
}
