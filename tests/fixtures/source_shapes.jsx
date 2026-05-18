export function Card({ title }) {
    return <section data-title={title}>{title}</section>;
}

export const Shell = () => {
    return <Card title="ready" />;
};
