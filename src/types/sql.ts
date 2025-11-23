export interface SqlRecord {
  id?: number;
  sqlText: string;
  sqlType?: string;
  databaseName?: string;
  executedAt?: string;
  executionSource?: string;
  isFavorite: boolean;
  tags?: string;
  description?: string;
  usageCount?: number;
  createdAt?: string;
}
