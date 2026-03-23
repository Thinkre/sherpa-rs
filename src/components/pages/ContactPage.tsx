import React from "react";
import { useTranslation } from "react-i18next";
import { Mail, Github } from "lucide-react";

export const ContactPage: React.FC = () => {
  const { t } = useTranslation();

  const contactMethods = [
    {
      icon: Github,
      title: t("contact.github"),
      description: t("contact.githubDescription"),
      action: () => {
        window.open("https://github.com/handy-computer/handy", "_blank");
      },
    },
    {
      icon: Mail,
      title: t("contact.email"),
      description: t("contact.emailDescription"),
      action: () => {
        window.open("mailto:support@handy.computer", "_blank");
      },
    },
  ];

  return (
    <div className="max-w-2xl">
      <h1 className="text-2xl font-bold mb-2">{t("contact.title")}</h1>
      <p className="text-sm text-mid-gray mb-8">{t("contact.description")}</p>

      <div className="space-y-4">
        {contactMethods.map((method, index) => {
          const Icon = method.icon;
          return (
            <div
              key={index}
              className="border border-mid-gray/20 rounded-lg p-6 hover:border-mid-gray/40 transition-colors cursor-pointer"
              onClick={method.action}
            >
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-lg bg-logo-primary/10 flex items-center justify-center shrink-0">
                  <Icon className="w-6 h-6 text-logo-primary" />
                </div>
                <div>
                  <h3 className="font-semibold mb-1">{method.title}</h3>
                  <p className="text-sm text-mid-gray">{method.description}</p>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
