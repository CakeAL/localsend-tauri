export interface DeviceMessage {
    alias: string,
    deviceModel: string,
    deviceType: string,
    download: boolean,
    fingerprint: string,
    port: number,
    protocol: string,
    version: string,
}