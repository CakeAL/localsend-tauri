export function showFileSize(size: number): string {
    if (size < 1024) {
      return `${size} Bytes`;
    } else if (size < Math.pow(1024, 2)) {
      return `${(size / 1024).toFixed(2)} KB`;
    } else if (size < Math.pow(1024, 3)) {
      return `${(size / Math.pow(1024, 2)).toFixed(2)} MB`;
    } else if (size < Math.pow(1024, 4)) {
      return `${(size / Math.pow(1024, 3)).toFixed(2)} GB`;
    } else {
      return `${(size / Math.pow(1024, 4)).toFixed(2)} TB`;
    }
  }