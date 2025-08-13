/*********************************************************/

SELECT CAST(1234 AS BINARY(2)) --?

/*

Important! 
- SQL Server stores INT and other numeric data types in byte-swapped order (little-endian) because it runs on Intel x86/x64 architectures
- But! SQL Server treats CAST(1234 AS BINARY(2)) as a direct conversion of INT value into a binary, not swapped (big-endian)


1. 1234 in binary (2 bytes):
1234 = 1024 + 128 + 64 + 16 + 2 → 00000100 11010010 

2. Binary to Hex (for readability):
Group into bytes: 00000100 11010010  →  04 D2 - This is big-endian representation

3. SQL Server stores INT in little-endian:
Reversed byte order: 04 D2 → D2 04 

So, the 2-byte little-endian BINARY(2) representation of 1234 is: 0xD204
But 2-byte big-endian SELECT CONVERT(BINARY(2), 1234) is		: 0x04D2
*/

SELECT 
	SUBSTRING(0xD204, 1, 2),
	CONVERT(BINARY(2), REVERSE(SUBSTRING(0xD204, 1, 2))), 
	CONVERT(INT, CONVERT(BINARY(2), REVERSE(SUBSTRING(0xD204, 1, 2))))


/*********************************************************/
--0. dbo.UDF_GetIntFromNext2Bytes

SELECT dbo.UDF_GetIntFromNext2Bytes(0xD204, 1) 

/*********************************************************/

--1. Fixed
--DROP TABLE IF EXISTS dbo.Fixed

CREATE TABLE dbo.Fixed
(
Col1 char(5) NOT NULL,
Col2 int NOT NULL,
Col3 char(3) NULL,
Col4 char(6) NOT NULL
);

INSERT dbo.Fixed 
VALUES ('ABCDE', 123, NULL, 'CCCC');

DELETE FROM dbo.Fixed


EXEC [dbo].[ResurrectionOfLostData]
	@SchemaTableName = 'dbo.Fixed'
	,@HasBlobData = 0
	,@IsDebugMode = 1



-- SELECT * FROM dbo.Fixed
/*********************************************************/

--2. Var
--DROP TABLE IF EXISTS dbo.Variable

CREATE TABLE dbo.Variable
(
Col1 char(3) NOT NULL,
Col2 varchar(250) NOT NULL,
Col3 varchar(5) NULL,
Col4 varchar(20) NOT NULL,
Col5 bit NULL,
Col6 smallint NULL,
Col7 DATE NOT NULL,
Col8 numeric(18,4) NULL,
Col9 FLOAT NOT NULL
);
INSERT dbo.Variable 
VALUES ('AAA', REPLICATE('X', 250), NULL, 'ABC', NULL, 123, '2025-08-21', 8.88, 9.9999);

DELETE FROM dbo.Variable

EXEC [dbo].[ResurrectionOfLostData]
	@SchemaTableName = 'dbo.Variable'
	,@HasBlobData = 0
	,@IsDebugMode = 1

-- SELECT * FROM dbo.Variable

/*********************************************************/
