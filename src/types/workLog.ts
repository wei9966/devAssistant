export interface WorkLog {
  id?: number;
  date: string;
  logType: string;
  content: string;
  aiGenerated: boolean;
  createdAt?: string;
  updatedAt?: string;
}
