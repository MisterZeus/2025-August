-- Demo Queries

-- 0- Environment Cleanup
:Connect sql-node3
USE [master];
IF EXISTS (SELECT 1 FROM sys.server_principals WHERE [name] = 'inst_login')
DROP LOGIN inst_login;
USE msdb;
SELECT * FROM sysjobs;
IF EXISTS (SELECT 1 FROM sysjobs WHERE [name] = 'inst_job')
EXEC dbo.sp_delete_job @job_name = N'inst_job';
GO
:Connect sql-node4
USE [master];
IF EXISTS (SELECT 1 FROM sys.server_principals WHERE [name] = 'inst_login')
DROP LOGIN inst_login;
USE msdb;
SELECT * FROM sysjobs;
IF EXISTS (SELECT 1 FROM sysjobs WHERE [name] = 'inst_job')
EXEC dbo.sp_delete_job @job_name = N'inst_job';
GO
:Connect cag-listener
USE [master];
IF EXISTS (SELECT 1 FROM sys.server_principals WHERE [name] = 'cag_login')
DROP LOGIN cag_login;
USE msdb;
IF EXISTS (SELECT 1 FROM sysjobs WHERE [name] = 'cag_job')
EXEC dbo.sp_delete_job @job_name = N'cag_job';
GO

-- 1- The logical vs. physical contained system databases
:Connect sql-node3
SELECT db.name, physical_database_name, physical_name 
FROM sys.databases db INNER JOIN sys.master_files mf ON db.database_id = mf.database_id
WHERE db.name LIKE '%msdb' OR db.name LIKE '%master';
GO

-- 2- Contained AG metadata
-- AG catalog views
:Connect sql-node3
SELECT [name] AS ag_name, CASE is_contained WHEN 0 THEN 'Normal' WHEN 1 THEN 'Contained' ELSE 'UNKNOWN' END AS ag_type 
FROM sys.availability_groups; 
GO
-- Sessions DMVs
:Connect sql-node3
SELECT [session_id], [host_name], [program_name] ,login_name, [status], DB_NAME(database_id) AS [database_name], contained_availability_group_id 
FROM sys.dm_exec_sessions
WHERE contained_availability_group_id IS NOT NULL;
GO

-- 3- Instance Logins vs. Contained AG logins
-- 3.1- Create a new login on instance-level
:Connect sql-node3
IF NOT EXISTS (SELECT 1 FROM sys.server_principals WHERE [name] = 'inst_login')
CREATE LOGIN inst_login with password='1234', default_database=[master], check_expiration=off, check_policy=off;
ELSE
SELECT 'Login exists!!' AS output_message;
GO
-- 3.2- Create a new login on ag-level
:Connect cag-listener
IF NOT EXISTS (SELECT 1 FROM sys.server_principals WHERE [name] = 'cag_login')
CREATE LOGIN cag_login with password='1234', default_database=[master], check_expiration=off, check_policy=off;
ELSE
SELECT 'Login exists!!' AS output_message;
GO
-- 3.3- Let us query sys.server_principals on node level
:Connect sql-node3
SELECT * 
FROM master.sys.server_principals
WHERE create_date > '2025-08-12 00:00:00.000';
GO
-- 3.3- Let us query sys.server_principals on ag level
:Connect cag-listener
SELECT * 
FROM master.sys.server_principals
WHERE create_date > '2025-08-12 00:00:00.000';
GO
-- 3.4- Let us connect using cag_login login
:Connect cag-listener
EXECUTE AS LOGIN = 'cag_login'
SELECT @@SERVERNAME AS server_name, SUSER_NAME() AS login_name;   
REVERT;
GO
-- 3.5- Let us test connecting using cag_login login after failing over
-- 3.5.1- Performing failover
:Connect sql-node4
ALTER AVAILABILITY GROUP [cag] FAILOVER;
GO
-- 3.5.2- Test ag status after failover
:Connect sql-node4
EXEC sp_get_ag_status @ag_name = 'cag';
GO
-- 3.5.3- Test connecting after failover
:Connect cag-listener
EXECUTE AS LOGIN = 'cag_login'
SELECT @@SERVERNAME AS server_name, SUSER_NAME() AS login_name;   
REVERT;
GO
 
-- 4- Instance Jobs vs. Contained AG Jobs
-- 4.1- Confirm the current primary node
:Connect sql-node4
EXEC sp_get_ag_primary_node @ag_name = 'cag';
GO
-- 4.2- Create a job on instance level
:Connect sql-node4
USE msdb;
IF NOT EXISTS (SELECT 1 FROM sysjobs WHERE [name] = 'inst_job')
EXEC dbo.sp_add_job @job_name = N'inst_job';
ELSE
SELECT 'Job already exists!!' AS output_message;
GO
-- 4.3- Create a job on ag level
:Connect cag-listener
USE msdb;
IF NOT EXISTS (SELECT 1 FROM sysjobs WHERE [name] = 'cag_job')
EXEC dbo.sp_add_job @job_name = N'cag_job';
ELSE
SELECT 'Job already exists!!' AS output_message;
GO
-- 4.4- Check created job on instance level
:Connect sql-node4
USE msdb;
SELECT * FROM sysjobs;
GO
-- 4. 5- Check created job on ag level
:Connect cag-listener
USE msdb;
SELECT * FROM sysjobs;
GO

-- 5- Instance backups vs. ag backpus
-- 5.1- Take a backup from instance level for ag databases [DemoDB_1]
:Connect sql-node4
IF NOT EXISTS (SELECT * FROM msdb.dbo.backupmediafamily WHERE physical_device_name = '\\ad-vm\SQL_Shared\DemoDB_1_instance.bak')
BACKUP DATABASE [DemoDB_1] TO DISK = '\\ad-vm\SQL_Shared\DemoDB_1_instance.bak' WITH FORMAT;
ELSE
SELECT 'Backup file already exists' AS output_message;
GO
-- 5.2- Take a backup from ag level for ag databases [DemoDB_1]
:Connect cag-listener
IF NOT EXISTS (SELECT * FROM msdb.dbo.backupmediafamily WHERE physical_device_name = '\\ad-vm\SQL_Shared\DemoDB_1_ag.bak')
BACKUP DATABASE [DemoDB_1] TO DISK = '\\ad-vm\SQL_Shared\DemoDB_1_ag.bak' WITH FORMAT;
ELSE
SELECT 'Backup file already exists' AS output_message;
GO
-- 5.3- Check backup history on the instance level
:Connect sql-node4
SELECT TOP 1 database_name, type, backup_start_date, backup_finish_date, physical_device_name
FROM msdb.dbo.backupset bs
JOIN msdb.dbo.backupmediafamily bmf ON bs.media_set_id = bmf.media_set_id
WHERE bs.database_name = 'DemoDB_1'
ORDER BY bs.backup_finish_date DESC;
GO
-- 5.4- Check backup history on the ag level
:Connect cag-listener
SELECT TOP 1 database_name, type, backup_start_date, backup_finish_date, physical_device_name
FROM msdb.dbo.backupset bs
JOIN msdb.dbo.backupmediafamily bmf ON bs.media_set_id = bmf.media_set_id
WHERE bs.database_name = 'DemoDB_1'
ORDER BY bs.backup_finish_date DESC;
GO

-- 6- Contained AG limitations (These will fail)
-- 6.1- Attempt to create a database from the ag level
:Connect cag-listener
CREATE DATABASE [DemoDB_2];
GO -- This fails with a message like: "CREATE DATABASE statement not allowed within a contained availability group."
-- 6.2- Attempt to restore a database from the AG listener
:Connect cag-listener
RESTORE DATABASE [DemoDB_2] FROM DISK = '\\ad-vm\SQL_Shared\DemoDB_1_ag.bak';
GO -- This also fails with a similar error.
-- 6.3- Attempt to perform a failover from the AG listener
:Connect cag-listener
ALTER AVAILABILITY GROUP [cag] FAILOVER;
GO -- This fails because failover is an instance-level command.

:Connect cag-listener
create login ag_admin with password= 'ag_admin', default_database=[master], check_expiration=off, check_policy=off
EXEC master..sp_addsrvrolemember @loginame = N'ag_admin', @rolename = N'sysadmin'
GO

-- 7- sysadmin access
-- 7.1- Checking local databases data
:Connect sql-node3
USE LocalDB_1
IF NOT EXISTS (SELECT 1 FROM sys.tables WHERE [name] = 'test')
BEGIN
	CREATE TABLE test (id int, name char(10));
	INSERT INTO test VALUES (1, 'Nader');
	SELECT * FROM test;
END
ELSE
SELECT * FROM test;
GO
-- 7.2- sysadmin account can access local databases
:Connect cag-listener
EXECUTE AS LOGIN = 'sa';
USE LocalDB_1
SELECT name FROM sys.tables;
SELECT * FROM test;
GO
:Connect cag-listener
EXECUTE AS LOGIN = 'sa';
USE LocalDB_1
DELETE FROM test;
DROP TABLE test;
GO