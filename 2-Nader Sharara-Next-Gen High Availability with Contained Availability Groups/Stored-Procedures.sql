-- Stored Procedures

/* 1- sp_get_ag_status SP*/
CREATE PROCEDURE sp_get_ag_status
@ag_name sysname

AS
SELECT 
        ag.name AS AgName,
        SUBSTRING(ar.replica_server_name,LEN(ar.replica_server_name),1) as [NodeId],
        -- ar.replica_server_name AS [Replica],
        ars.role_desc AS [Role],
        ars.connected_state_desc AS [ConnectState],
        ars.synchronization_health_desc AS [SyncHealth],
        ars.operational_state_desc AS [OperatState],
        COUNT(drs.database_id) AS [DbCount],
        SUM(CASE WHEN drs.synchronization_state_desc = 'SYNCHRONIZED' THEN 1 ELSE 0 END) AS [SyncDBs],
        SUM(CASE WHEN drs.synchronization_state_desc <> 'SYNCHRONIZED' THEN 1 ELSE 0 END) AS [NotSyncDBs],
        MAX(CASE 
        WHEN ars.role_desc = 'SECONDARY' AND drs.last_hardened_time IS NOT NULL 
        THEN DATEDIFF(SECOND, drs.last_hardened_time, GETDATE()) 
        ELSE NULL 
        END) AS [MaxLatencySec],
        ar.availability_mode_desc AS [SyncMode],
        ar.failover_mode_desc AS [FailoverMode],
        ar.backup_priority AS [BackupPriority],
        ar.secondary_role_allow_connections_desc AS [ReadableSecondary]
FROM sys.availability_groups ag
        JOIN sys.availability_replicas ar ON ag.group_id = ar.group_id
        JOIN sys.dm_hadr_availability_replica_states ars ON ar.replica_id = ars.replica_id
        LEFT JOIN sys.dm_hadr_database_replica_states drs ON ars.replica_id = drs.replica_id AND ag.group_id = drs.group_id
WHERE ag.name = @ag_name
GROUP BY 
        ag.name,
        ar.replica_server_name,
        ars.role_desc,
        ar.availability_mode_desc,
        ar.failover_mode_desc,
        ars.connected_state_desc,
        ars.synchronization_health_desc,
        ars.operational_state_desc,
        ar.backup_priority,
        ar.secondary_role_allow_connections_desc,
        ag.group_id
ORDER BY 
        ag.name,
        CASE WHEN ars.role_desc = 'PRIMARY' THEN 0 ELSE 1 END,
        ar.replica_server_name;
SELECT 
        ag.name AS [AgName],
        SUBSTRING(ar.replica_server_name,LEN(ar.replica_server_name),1) as [NodeId],
        -- ar.replica_server_name AS [Replica],
        ars.role_desc AS [Role],
        COALESCE(adc.database_name, DB_NAME(drs.database_id)) AS [Database],
        drs.synchronization_state_desc AS [SyncState],
        drs.synchronization_health_desc AS [DBHealth],
        drs.is_suspended AS [IsSuspended],
        ISNULL(drs.suspend_reason_desc, 'N/A') AS [SuspendReason],
        drs.last_hardened_time AS [LastHardenedTime],
        drs.last_commit_time AS [LastCommitTime],
        drs.log_send_queue_size AS [SendQueueKB],
        drs.log_send_rate AS [SendRateKbSec],
        drs.redo_queue_size AS [RedoQueueKB],
        drs.redo_rate AS [RedoRateKbSec],
        CASE 
        WHEN ars.role_desc = 'PRIMARY' OR drs.last_hardened_time IS NULL THEN NULL 
        ELSE DATEDIFF(SECOND, drs.last_hardened_time, GETDATE()) 
        END AS [LatencySec],
        CASE
        WHEN ars.role_desc = 'PRIMARY'
        THEN ''
        WHEN ars.role_desc = 'SECONDARY'  
        AND ars.connected_state_desc = 'CONNECTED' 
        AND drs.synchronization_state_desc = 'SYNCHRONIZED' 
        THEN 'Failover Ready'
        ELSE 'Not Ready'
        END AS [FailoverReadiness]
FROM sys.availability_groups ag
        JOIN sys.availability_replicas ar ON ag.group_id = ar.group_id
        JOIN sys.dm_hadr_availability_replica_states ars ON ar.replica_id = ars.replica_id
        JOIN sys.dm_hadr_database_replica_states drs ON ag.group_id = drs.group_id AND drs.replica_id = ars.replica_id
        LEFT JOIN sys.availability_databases_cluster adc ON ag.group_id = adc.group_id AND drs.group_database_id = adc.group_database_id
WHERE ag.name = @ag_name
ORDER BY 
        ag.name,
        CASE WHEN ars.role_desc = 'PRIMARY' THEN 0 ELSE 1 END,
        ar.replica_server_name,
        COALESCE(adc.database_name, DB_NAME(drs.database_id));
GO

/* 2- sp_get_ag_primary_node SP*/
CREATE PROCEDURE sp_get_ag_primary_node
@ag_name sysname
AS
SELECT	LOWER(dhags.primary_replica) AS primary_replica_name
FROM	sys.availability_groups AS ag
	INNER JOIN  sys.dm_hadr_availability_group_states dhags
	ON (ag.group_id = dhags.group_id)
WHERE ag.name = @ag_name;
GO

/* 3- sp_get_ag_details SP*/
CREATE PROCEDURE sp_get_ag_details
@ag_name sysname
AS
SELECT	ag.name, LOWER(dhags.primary_replica) AS primary_replica_name, ag.cluster_type_desc, ag.is_contained, ag.is_distributed
FROM	sys.availability_groups AS ag
	INNER JOIN  sys.dm_hadr_availability_group_states dhags
	ON (ag.group_id = dhags.group_id)
WHERE ag.name = @ag_name;
GO

