import React from "react";

export interface PanelProps<T> {
  title: string;
  items: T[];
  render(item: T): React.ReactNode;
}

export function Panel<T extends { id: string }>(props: PanelProps<T>) {
  function Item({ item }: { item: T }) {
    return <li>{props.render(item)}</li>;
  }

  return <section aria-label={props.title}><ul>{props.items.map((item) => <Item key={item.id} item={item} />)}</ul></section>;
}

export const Toolbar = ({ onRun }: { onRun(): void }) => {
  const handle = () => onRun();
  return <button onClick={handle}>Run</button>;
};
