import React, { useMemo } from "react";

export function Dashboard({ rows, render }) {
  function Row({ row }) {
    const cells = row.items.map((item) => render?.(item) ?? <span>{item}</span>);
    return <section>{cells}</section>;
  }

  return <main>{rows.map((row) => <Row key={row.id} row={row} />)}</main>;
}

export const Shell = ({ children }) => {
  const wrapped = useMemo(() => <Dashboard rows={[]} render={() => children} />, [children]);
  return <div>{wrapped}</div>;
};
