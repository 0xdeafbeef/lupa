WITH RECURSIVE
    -- walking graph in up direction --
    root_traversal AS (
        -- getting current node--
        (SELECT message_hash
         FROM transaction_messages
         WHERE transaction_hash IN (SELECT transaction_hash
                                    FROM transaction_messages
                                    WHERE message_hash = ?
                                      AND is_out)
           AND !is_out
         LIMIT 1)
        UNION ALL
        -- walking in the up direction from previous node --
        SELECT tm.message_hash
        FROM transaction_messages AS tm
                 INNER JOIN transaction_messages tm1
                            ON tm.transaction_hash = tm1.transaction_hash
                 INNER JOIN root_traversal
                            ON tm1.message_hash = root_traversal.message_hash
        WHERE !tm.is_out
          AND tm1.is_out
        LIMIT 10000),
    child_traversal AS (
        -- getting root node --
        SELECT IF(COUNT(root_traversal.message_hash) >= 1,
                  root_traversal.message_hash, ?) AS message_hash
        FROM root_traversal
                 INNER JOIN messages m
                            ON m.message_hash = root_traversal.message_hash AND
                               message_type =
                               'ExternalIn' #TODO: check if this is correct in case of TickTock
        UNION
        -- walking from the root node down to the child --
        SELECT tm.message_hash
        FROM transaction_messages AS tm
                 INNER JOIN transaction_messages tm1
                            ON tm.transaction_hash = tm1.transaction_hash
                 INNER JOIN child_traversal
                            ON tm1.message_hash = child_traversal.message_hash
        WHERE tm.is_out
          AND !tm1.is_out
        LIMIT 10000)

SELECT /*+  MAX_EXECUTION_TIME(10000) */
    tm.message_hash         AS message_hash,
    tm.index_in_transaction AS index_in_transaction,
    tm.is_out               AS is_out,
    transaction_time,
    m.dst_workchain,
    m.dst_address,
    m.src_workchain,
    m.src_address,

    tm.transaction_hash     AS transaction_hash,
    m.message_type,
    m.message_value,
    m.bounced,
    m.bounce,
    parsed_id,
    parsed_type,
    method_name,

    c.contract_name         AS contract_name,

    t.workchain,
    t.account_id,
    t.lt,
    t.time,
    t.hash,
    t.block_shard,
    t.block_seqno,
    t.block_hash,
    t.tx_type,
    t.aborted,
    t.balance_change,
    t.exit_code,
    t.result_code
FROM transaction_messages tm
         INNER JOIN child_traversal
                    ON tm.message_hash = child_traversal.message_hash
         LEFT JOIN parsed_messages_new pm ON tm.message_hash = pm.message_hash
         LEFT JOIN accounts a
                   ON dst_address = a.address AND
                      dst_workchain = a.workchain
         LEFT JOIN contracts_info c ON c.code_hash = a.code_hash
         INNER JOIN transactions t ON t.hash = tm.transaction_hash
         INNER JOIN messages m ON tm.message_hash = m.message_hash
ORDER BY m.created_at, m.created_lt
LIMIT 10000;
