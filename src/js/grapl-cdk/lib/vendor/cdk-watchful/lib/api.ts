import cloudwatch = require('@aws-cdk/aws-cloudwatch');

export interface IWatchful {
  addSection(title: string, options?: SectionOptions): void
  addAlarm(alarm: cloudwatch.Alarm): void;
  addWidgets(...widgets: cloudwatch.IWidget[]): void;
}

export interface SectionOptions {
  readonly links?: QuickLink[];
}

export interface QuickLink {
  readonly title: string;
  readonly url: string;
}

